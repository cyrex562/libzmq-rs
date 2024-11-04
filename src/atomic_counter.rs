use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

pub struct AtomicCounter {
    value: AtomicU32,
    #[cfg(feature = "use_mutex")]
    sync: Mutex<()>,
}

impl AtomicCounter {
    pub fn new(value: u32) -> Self {
        Self {
            value: AtomicU32::new(value),
            #[cfg(feature = "use_mutex")]
            sync: Mutex::new(()),
        }
    }

    pub fn set(&self, value: u32) {
        self.value.store(value, Ordering::SeqCst);
    }

    pub fn add(&self, increment: u32) -> u32 {
        #[cfg(feature = "use_mutex")]
        {
            let _lock = self.sync.lock().unwrap();
            let old_value = self.value.load(Ordering::SeqCst);
            self.value.store(old_value + increment, Ordering::SeqCst);
            old_value
        }
        #[cfg(not(feature = "use_mutex"))]
        {
            self.value.fetch_add(increment, Ordering::AcqRel)
        }
    }

    pub fn sub(&self, decrement: u32) -> bool {
        #[cfg(feature = "use_mutex")]
        {
            let _lock = self.sync.lock().unwrap();
            let old_value = self.value.load(Ordering::SeqCst);
            self.value.store(old_value - decrement, Ordering::SeqCst);
            old_value != decrement
        }
        #[cfg(not(feature = "use_mutex"))]
        {
            self.value.fetch_sub(decrement, Ordering::AcqRel) != decrement
        }
    }

    pub fn get(&self) -> u32 {
        self.value.load(Ordering::SeqCst)
    }
}
