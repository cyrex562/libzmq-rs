use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};

// Forward declarations
pub struct Context;
pub struct IoThread;
pub struct Options {
    pub linger: AtomicU64,
}

// Base trait for objects (replaces object_t)
pub trait Object {
    fn process_plug(&mut self, object: &mut Own);
    fn process_term(&mut self, linger: i32);
    fn send_term(&mut self, object: &mut Own, linger: i32);
    fn send_term_ack(&mut self, owner: &mut Own);
    fn send_term_req(&mut self, owner: &mut Own, object: &mut Own);
    fn send_own(&mut self, owner: &mut Own, object: &mut Own);
}

pub struct Own {
    options: Options,
    terminating: bool,
    sent_seqnum: AtomicU64,
    processed_seqnum: u64,
    owner: Option<*mut Own>,  // Using raw pointer for owner to avoid reference cycles
    owned: HashSet<*mut Own>, // Using raw pointer for owned objects
    term_acks: i32,
}

impl Own {
    pub fn new_with_context(parent: &mut Context, tid: u32) -> Self {
        Self {
            options: Options { linger: AtomicU64::new(0) },
            terminating: false,
            sent_seqnum: AtomicU64::new(0),
            processed_seqnum: 0,
            owner: None,
            owned: HashSet::new(),
            term_acks: 0,
        }
    }

    pub fn new_with_io_thread(io_thread: &mut IoThread, options: Options) -> Self {
        Self {
            options,
            terminating: false,
            sent_seqnum: AtomicU64::new(0),
            processed_seqnum: 0,
            owner: None,
            owned: HashSet::new(),
            term_acks: 0,
        }
    }

    pub fn set_owner(&mut self, owner: *mut Own) {
        assert!(self.owner.is_none());
        self.owner = Some(owner);
    }

    pub fn inc_seqnum(&self) {
        self.sent_seqnum.fetch_add(1, Ordering::SeqCst);
    }

    pub fn process_seqnum(&mut self) {
        self.processed_seqnum += 1;
        self.check_term_acks();
    }

    pub fn launch_child(&mut self, object: *mut Own) {
        unsafe {
            (*object).set_owner(self);
            self.process_plug(&mut *object);
            self.send_own(self, &mut *object);
        }
    }

    pub fn term_child(&mut self, object: &mut Own) {
        self.process_term_req(object);
    }

    pub fn process_term_req(&mut self, object: *mut Own) {
        if self.terminating {
            return;
        }

        if !self.owned.remove(&object) {
            return;
        }

        self.register_term_acks(1);
        self.send_term(unsafe { &mut *object }, self.options.linger.load(Ordering::SeqCst) as i32);
    }

    pub fn process_own(&mut self, object: *mut Own) {
        if self.terminating {
            self.register_term_acks(1);
            self.send_term(unsafe { &mut *object }, 0);
            return;
        }

        self.owned.insert(object);
    }

    pub fn terminate(&mut self) {
        if self.terminating {
            return;
        }

        if self.owner.is_none() {
            self.process_term(self.options.linger.load(Ordering::SeqCst) as i32);
            return;
        }

        if let Some(owner) = self.owner {
            unsafe {
                self.send_term_req(&mut *owner, self);
            }
        }
    }

    pub fn is_terminating(&self) -> bool {
        self.terminating
    }

    pub fn process_term(&mut self, linger: i32) {
        assert!(!self.terminating);

        let owned_count = self.owned.len();
        for object in self.owned.iter() {
            unsafe {
                self.send_term(&mut **object, linger);
            }
        }
        self.register_term_acks(owned_count as i32);
        self.owned.clear();

        self.terminating = true;
        self.check_term_acks();
    }

    pub fn register_term_acks(&mut self, count: i32) {
        self.term_acks += count;
    }

    pub fn unregister_term_ack(&mut self) {
        assert!(self.term_acks > 0);
        self.term_acks -= 1;
        self.check_term_acks();
    }

    pub fn process_term_ack(&mut self) {
        self.unregister_term_ack();
    }

    fn check_term_acks(&mut self) {
        if self.terminating 
            && self.processed_seqnum == self.sent_seqnum.load(Ordering::SeqCst)
            && self.term_acks == 0 
        {
            assert!(self.owned.is_empty());

            if let Some(owner) = self.owner {
                unsafe {
                    self.send_term_ack(&mut *owner);
                }
            }

            self.process_destroy();
        }
    }

    fn process_destroy(&mut self) {
        // In Rust, this would be handled by Drop
    }
}

// Implement Drop instead of destructor
impl Drop for Own {
    fn drop(&mut self) {
        // Cleanup code here if needed
    }
}

// Implement Object trait for Own
impl Object for Own {
    fn process_plug(&mut self, _object: &mut Own) {
        // Implementation here
    }

    fn process_term(&mut self, linger: i32) {
        // Implementation here
    }

    fn send_term(&mut self, _object: &mut Own, _linger: i32) {
        // Implementation here
    }

    fn send_term_ack(&mut self, _owner: &mut Own) {
        // Implementation here
    }

    fn send_term_req(&mut self, _owner: &mut Own, _object: &mut Own) {
        // Implementation here
    }

    fn send_own(&mut self, _owner: &mut Own, _object: &mut Own) {
        // Implementation here
    }
}
