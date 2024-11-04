use std::collections::HashSet;
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug)]
pub struct GenericMtrie<T> {
    pipes: Option<HashSet<T>>,
    num_prefixes: AtomicU32,
    min: u8,
    count: u16,
    live_nodes: u16,
    next: NextNode<T>,
}

#[derive(Debug)]
enum NextNode<T> {
    Empty,
    Single(Box<GenericMtrie<T>>),
    Table(Vec<Option<Box<GenericMtrie<T>>>>),
}

#[derive(PartialEq)]
pub enum RemoveResult {
    NotFound,
    LastValueRemoved,
    ValuesRemain,
}

impl<T: Eq + std::hash::Hash + Clone> GenericMtrie<T> {
    pub fn new() -> Self {
        GenericMtrie {
            pipes: None,
            num_prefixes: AtomicU32::new(0),
            min: 0,
            count: 0,
            live_nodes: 0,
            next: NextNode::Empty,
        }
    }

    pub fn add(&mut self, prefix: &[u8], value: T) -> bool {
        let mut current = self;
        let mut prefix_iter = prefix.iter();

        while let Some(&c) = prefix_iter.next() {
            if c < current.min || c >= current.min + current.count as u8 {
                // Extend the table
                if current.count == 0 {
                    current.min = c;
                    current.count = 1;
                    current.next = NextNode::Empty;
                } else {
                    current.extend_table(c);
                }
            }

            current = match &mut current.next {
                NextNode::Empty => {
                    let new_node = Box::new(GenericMtrie::new());
                    current.next = NextNode::Single(new_node);
                    current.live_nodes += 1;
                    match &mut current.next {
                        NextNode::Single(node) => node,
                        _ => unreachable!(),
                    }
                }
                NextNode::Single(node) if current.count == 1 => node,
                NextNode::Table(table) => {
                    let idx = (c - current.min) as usize;
                    if table[idx].is_none() {
                        table[idx] = Some(Box::new(GenericMtrie::new()));
                        current.live_nodes += 1;
                    }
                    table[idx].as_mut().unwrap()
                }
                _ => unreachable!(),
            };
        }

        let is_new = current.pipes.is_none();
        if is_new {
            current.pipes = Some(HashSet::new());
            self.num_prefixes.fetch_add(1, Ordering::SeqCst);
        }
        current.pipes.as_mut().unwrap().insert(value);
        is_new
    }

    pub fn remove(&mut self, prefix: &[u8], value: &T) -> RemoveResult {
        if prefix.is_empty() {
            if let Some(pipes) = &mut self.pipes {
                if pipes.remove(value) {
                    if pipes.is_empty() {
                        self.pipes = None;
                        self.num_prefixes.fetch_sub(1, Ordering::SeqCst);
                        return RemoveResult::LastValueRemoved;
                    }
                    return RemoveResult::ValuesRemain;
                }
            }
            return RemoveResult::NotFound;
        }

        // Handle non-empty prefix traversal and removal
        let mut result = RemoveResult::NotFound;
        let c = prefix[0];
        
        if let Some(next) = self.get_next_node(c) {
            result = next.remove(&prefix[1..], value);
            if next.is_redundant() {
                self.remove_child_node(c);
            }
        }

        result
    }

    pub fn match_prefix<F>(&self, data: &[u8], mut callback: F)
    where
        F: FnMut(&T),
    {
        let mut current = self;
        let mut data_iter = data.iter();

        loop {
            if let Some(pipes) = &current.pipes {
                for pipe in pipes {
                    callback(pipe);
                }
            }

            match data_iter.next() {
                None => break,
                Some(&c) if current.count == 0 => break,
                Some(&c) => {
                    match &current.next {
                        NextNode::Single(node) if current.count == 1 && c == current.min => {
                            current = node;
                        }
                        NextNode::Table(table) if c >= current.min 
                            && c < current.min + current.count as u8 => {
                            if let Some(Some(node)) = table.get((c - current.min) as usize) {
                                current = node;
                            } else {
                                break;
                            }
                        }
                        _ => break,
                    }
                }
            }
        }
    }

    pub fn num_prefixes(&self) -> u32 {
        self.num_prefixes.load(Ordering::SeqCst)
    }

    fn is_redundant(&self) -> bool {
        self.pipes.is_none() && self.live_nodes == 0
    }

    fn extend_table(&mut self, c: u8) {
        let new_min = std::cmp::min(self.min, c);
        let new_max = std::cmp::max(self.min + self.count as u8 - 1, c);
        let new_count = (new_max - new_min + 1) as u16;

        let mut new_table = vec![None; new_count as usize];
        
        match std::mem::replace(&mut self.next, NextNode::Empty) {
            NextNode::Single(node) if self.count == 1 => {
                let idx = (self.min - new_min) as usize;
                new_table[idx] = Some(node);
            }
            NextNode::Table(old_table) => {
                for (i, node) in old_table.into_iter().enumerate() {
                    if let Some(node) = node {
                        new_table[(self.min - new_min) as usize + i] = Some(node);
                    }
                }
            }
            _ => {}
        }

        self.min = new_min;
        self.count = new_count;
        self.next = NextNode::Table(new_table);
    }

    fn get_next_node(&mut self, c: u8) -> Option<&mut GenericMtrie<T>> {
        if c < self.min || c >= self.min + self.count as u8 {
            return None;
        }

        match &mut self.next {
            NextNode::Single(node) if self.count == 1 => Some(node),
            NextNode::Table(table) => {
                let idx = (c - self.min) as usize;
                table.get_mut(idx)?.as_mut()
            }
            _ => None,
        }
    }

    fn remove_child_node(&mut self, c: u8) {
        self.live_nodes -= 1;
        match &mut self.next {
            NextNode::Single(_) if self.count == 1 => {
                self.next = NextNode::Empty;
                self.count = 0;
            }
            NextNode::Table(table) => {
                let idx = (c - self.min) as usize;
                table[idx] = None;
                
                if self.live_nodes == 1 {
                    // Compact to single node
                    let mut single_node = None;
                    for node in table.iter_mut() {
                        if let Some(n) = node.take() {
                            single_node = Some(n);
                            break;
                        }
                    }
                    if let Some(node) = single_node {
                        self.next = NextNode::Single(node);
                        self.count = 1;
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let mut mtrie = GenericMtrie::new();
        
        // Test add
        assert!(mtrie.add(b"test", 1));
        assert!(!mtrie.add(b"test", 2));
        
        // Test match
        let mut matches = Vec::new();
        mtrie.match_prefix(b"test", |&v| matches.push(v));
        assert_eq!(matches, vec![1, 2]);
        
        // Test remove
        assert_eq!(mtrie.remove(b"test", &1), RemoveResult::ValuesRemain);
        assert_eq!(mtrie.remove(b"test", &2), RemoveResult::LastValueRemoved);
    }
}
