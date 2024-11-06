use std::sync::atomic::{AtomicU32, Ordering};

enum NextNode {
    None,
    Single(Box<Trie>),
    Table(Vec<Option<Box<Trie>>>)
}

pub struct Trie {
    refcnt: u32,
    min: u8,
    count: u16,
    live_nodes: u16,
    next: NextNode,
}

pub struct TrieWithSize {
    num_prefixes: AtomicU32,
    trie: Trie,
}

impl Trie {
    pub fn new() -> Self {
        Trie {
            refcnt: 0,
            min: 0,
            count: 0,
            live_nodes: 0,
            next: NextNode::None,
        }
    }

    pub fn add(&mut self, prefix: &[u8]) -> bool {
        if prefix.is_empty() {
            self.refcnt += 1;
            return self.refcnt == 1;
        }

        let c = prefix[0];
        
        if self.count == 0 || c < self.min || c >= self.min + self.count as u8 {
            // Extend table
            if self.count == 0 {
                self.min = c;
                self.count = 1;
                self.next = NextNode::None;
            } else if self.count == 1 {
                let old_c = self.min;
                let old_node = match std::mem::replace(&mut self.next, NextNode::None) {
                    NextNode::Single(node) => node,
                    _ => unreachable!()
                };
                
                self.count = std::cmp::max(
                    (if self.min < c { c - self.min } else { self.min - c }) + 1,
                    1
                ) as u16;
                self.min = std::cmp::min(self.min, c);
                
                let mut table = vec![None; self.count as usize];
                table[(old_c - self.min) as usize] = Some(old_node);
                self.next = NextNode::Table(table);
            } else {
                match &mut self.next {
                    NextNode::Table(table) => {
                        if self.min < c {
                            let new_count = (c - self.min + 1) as u16;
                            table.resize(new_count as usize, None);
                            self.count = new_count;
                        } else {
                            let old_min = self.min;
                            self.min = c;
                            let new_count = (old_min + self.count as u8 - c) as u16;
                            let mut new_table = vec![None; new_count as usize];
                            new_table[(old_min - c) as usize..]
                                .clone_from_slice(&table[..]);
                            self.count = new_count;
                            self.next = NextNode::Table(new_table);
                        }
                    }
                    _ => unreachable!()
                }
            }
        }

        // Add node
        match &mut self.next {
            NextNode::None => {
                let mut new_node = Box::new(Trie::new());
                self.live_nodes += 1;
                let result = new_node.add(&prefix[1..]);
                self.next = NextNode::Single(new_node);
                result
            }
            NextNode::Single(ref mut node) => {
                node.add(&prefix[1..])
            }
            NextNode::Table(table) => {
                let idx = (c - self.min) as usize;
                if table[idx].is_none() {
                    table[idx] = Some(Box::new(Trie::new()));
                    self.live_nodes += 1;
                }
                table[idx].as_mut().unwrap().add(&prefix[1..])
            }
        }
    }

    pub fn check(&self, data: &[u8]) -> bool {
        let mut current = self;
        let mut data = data;

        loop {
            if current.refcnt > 0 {
                return true;
            }

            if data.is_empty() {
                return false;
            }

            let c = data[0];
            if c < current.min || c >= current.min + current.count as u8 {
                return false;
            }

            match &current.next {
                NextNode::Single(node) => {
                    current = node;
                }
                NextNode::Table(table) => {
                    if let Some(node) = &table[(c - current.min) as usize] {
                        current = node;
                    } else {
                        return false;
                    }
                }
                NextNode::None => return false,
            }

            data = &data[1..];
        }
    }

    fn is_redundant(&self) -> bool {
        self.refcnt == 0 && self.live_nodes == 0
    }
}

impl TrieWithSize {
    pub fn new() -> Self {
        TrieWithSize {
            num_prefixes: AtomicU32::new(0),
            trie: Trie::new(),
        }
    }

    pub fn add(&mut self, prefix: &[u8]) -> bool {
        if self.trie.add(prefix) {
            self.num_prefixes.fetch_add(1, Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    pub fn check(&self, data: &[u8]) -> bool {
        self.trie.check(data)
    }

    pub fn num_prefixes(&self) -> u32 {
        self.num_prefixes.load(Ordering::SeqCst)
    }
}
