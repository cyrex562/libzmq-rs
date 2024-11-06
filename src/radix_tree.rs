use std::mem;
use std::ptr;

#[derive(Debug)]
struct Node {
    refcount: u32,
    prefix: Vec<u8>,
    edges: Vec<Edge>,
}

#[derive(Debug)]
struct Edge {
    first_byte: u8,
    node: Box<Node>,
}

impl Node {
    fn new() -> Self {
        Node {
            refcount: 0,
            prefix: Vec::new(),
            edges: Vec::new(),
        }
    }

    fn with_prefix(prefix: &[u8]) -> Self {
        Node {
            refcount: 0,
            prefix: prefix.to_vec(),
            edges: Vec::new(),
        }
    }
}

#[derive(Debug)]
struct MatchResult {
    key_bytes_matched: usize,
    prefix_bytes_matched: usize,
    edge_index: Option<usize>,
    parent_edge_index: Option<usize>,
    current_node: *mut Node,
    parent_node: *mut Node,
    grandparent_node: *mut Node,
}

pub struct RadixTree {
    root: Box<Node>,
    size: usize,
}

impl RadixTree {
    pub fn new() -> Self {
        RadixTree {
            root: Box::new(Node::new()),
            size: 0,
        }
    }

    pub fn add(&mut self, key: &[u8]) -> bool {
        let match_result = self.match_key(key, false);
        let key_bytes_matched = match_result.key_bytes_matched;
        let prefix_bytes_matched = match_result.prefix_bytes_matched;
        
        unsafe {
            let current_node = &mut *match_result.current_node;
            
            if key_bytes_matched != key.len() {
                if prefix_bytes_matched == current_node.prefix.len() {
                    // Create new leaf node
                    let mut key_node = Node::with_prefix(&key[key_bytes_matched..]);
                    key_node.refcount = 1;
                    
                    // Add edge to new node
                    current_node.edges.push(Edge {
                        first_byte: key[key_bytes_matched],
                        node: Box::new(key_node),
                    });
                    
                    self.size += 1;
                    return true;
                }
                
                // Split node
                let mut key_node = Node::with_prefix(&key[key_bytes_matched..]);
                key_node.refcount = 1;
                
                let mut split_node = Node::with_prefix(&current_node.prefix[prefix_bytes_matched..]);
                split_node.refcount = current_node.refcount;
                split_node.edges = mem::replace(&mut current_node.edges, Vec::new());
                
                current_node.prefix.truncate(prefix_bytes_matched);
                current_node.refcount = 0;
                
                current_node.edges.push(Edge {
                    first_byte: key[key_bytes_matched],
                    node: Box::new(key_node),
                });
                
                current_node.edges.push(Edge {
                    first_byte: split_node.prefix[0],
                    node: Box::new(split_node),
                });
                
                self.size += 1;
                return true;
            }
            
            if prefix_bytes_matched != current_node.prefix.len() {
                let mut split_node = Node::with_prefix(&current_node.prefix[prefix_bytes_matched..]);
                split_node.refcount = current_node.refcount;
                split_node.edges = mem::replace(&mut current_node.edges, Vec::new());
                
                current_node.prefix.truncate(prefix_bytes_matched);
                current_node.refcount = 1;
                
                current_node.edges.push(Edge {
                    first_byte: split_node.prefix[0],
                    node: Box::new(split_node),
                });
                
                self.size += 1;
                return true;
            }
            
            current_node.refcount += 1;
            self.size += 1;
            current_node.refcount == 1
        }
    }

    pub fn remove(&mut self, key: &[u8]) -> bool {
        let match_result = self.match_key(key, false);
        
        unsafe {
            let current_node = &mut *match_result.current_node;
            
            if match_result.key_bytes_matched != key.len() 
               || match_result.prefix_bytes_matched != current_node.prefix.len()
               || current_node.refcount == 0 {
                return false;
            }
            
            current_node.refcount -= 1;
            self.size -= 1;
            
            if current_node.refcount > 0 {
                return false;
            }
            
            // TODO: Implement node merging logic for cleaner tree
            
            true
        }
    }

    pub fn contains(&self, key: &[u8]) -> bool {
        if self.root.refcount > 0 {
            return true;
        }
        
        let match_result = self.match_key(key, true);
        unsafe {
            let current_node = &*match_result.current_node;
            match_result.key_bytes_matched == key.len()
                && match_result.prefix_bytes_matched == current_node.prefix.len()
                && current_node.refcount > 0
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    fn match_key(&self, key: &[u8], is_lookup: bool) -> MatchResult {
        let mut current: *mut Node = &mut *self.root as *mut Node;
        let mut parent = current;
        let mut grandparent = current;
        let mut key_byte_index = 0;
        let mut edge_index = None;
        let mut parent_edge_index = None;

        unsafe {
            while key_byte_index < key.len() {
                let node = &*current;
                
                // Match prefix
                let mut prefix_matched = 0;
                while prefix_matched < node.prefix.len() && key_byte_index < key.len() {
                    if node.prefix[prefix_matched] != key[key_byte_index] {
                        break;
                    }
                    prefix_matched += 1;
                    key_byte_index += 1;
                }

                if is_lookup && prefix_matched == node.prefix.len() && node.refcount > 0 {
                    key_byte_index = key.len();
                    break;
                }

                if prefix_matched != node.prefix.len() || key_byte_index == key.len() {
                    break;
                }

                // Find matching edge
                let mut found_edge = false;
                for (i, edge) in node.edges.iter().enumerate() {
                    if edge.first_byte == key[key_byte_index] {
                        parent_edge_index = edge_index;
                        edge_index = Some(i);
                        grandparent = parent;
                        parent = current;
                        current = &mut *edge.node as *mut Node;
                        found_edge = true;
                        break;
                    }
                }

                if !found_edge {
                    break;
                }
            }

            MatchResult {
                key_bytes_matched: key_byte_index,
                prefix_bytes_matched: if key_byte_index < key.len() {
                    let node = &*current;
                    let mut i = 0;
                    while i < node.prefix.len() && key_byte_index + i < key.len() {
                        if node.prefix[i] != key[key_byte_index + i] {
                            break;
                        }
                        i += 1;
                    }
                    i
                } else {
                    (&*current).prefix.len()
                },
                edge_index,
                parent_edge_index,
                current_node: current,
                parent_node: parent,
                grandparent_node: grandparent,
            }
        }
    }
}

impl Drop for RadixTree {
    fn drop(&mut self) {
        // Rust's ownership system will handle cleanup automatically
    }
}
