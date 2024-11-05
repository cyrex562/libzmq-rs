use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Metadata {
    // Reference counter using atomic operations
    ref_cnt: AtomicUsize,
    // Dictionary holding metadata using Rust's HashMap
    dict: HashMap<String, String>,
}

impl Metadata {
    pub fn new(dict: HashMap<String, String>) -> Self {
        Metadata {
            ref_cnt: AtomicUsize::new(1),
            dict,
        }
    }

    // Returns Option<&str> instead of raw pointer
    pub fn get(&self, property: &str) -> Option<&str> {
        if property == "Identity" {
            // Handle legacy "Identity" property
            return self.get("ZMQ_MSG_PROPERTY_ROUTING_ID");
        }
        self.dict.get(property).map(|s| s.as_str())
    }

    pub fn add_ref(&self) {
        self.ref_cnt.fetch_add(1, Ordering::SeqCst);
    }

    // Returns true if the reference count drops to zero
    pub fn drop_ref(&self) -> bool {
        self.ref_cnt.fetch_sub(1, Ordering::SeqCst) == 1
    }
}

impl Drop for Metadata {
    fn drop(&mut self) {
        // Rust's drop implementation handles cleanup automatically
    }
}
