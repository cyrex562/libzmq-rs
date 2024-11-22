use std::sync::atomic::{AtomicI32, AtomicPtr, Ordering};
use std::sync::Mutex;

pub struct AtomicPtrWrapper<T> {
    ptr: AtomicPtr<T>,
    sync: Mutex<()>,
}

impl<T> AtomicPtrWrapper<T> {
    pub fn new() -> Self {
        Self {
            ptr: AtomicPtr::new(std::ptr::null_mut()),
            sync: Mutex::new(()),
        }
    }

    pub fn set(&self, ptr: *mut T) {
        self.ptr.store(ptr, Ordering::Relaxed);
    }

    pub fn xchg(&self, val: *mut T) -> *mut T {
        let _lock = self.sync.lock().unwrap();
        let old = self.ptr.load(Ordering::Relaxed);
        self.ptr.store(val, Ordering::Relaxed);
        old
    }

    pub fn cas(&self, cmp: *mut T, val: *mut T) -> *mut T {
        let _lock = self.sync.lock().unwrap();
        let old = self.ptr.load(Ordering::Relaxed);
        if old == cmp {
            self.ptr.store(val, Ordering::Relaxed);
        }
        old
    }
}

pub struct AtomicValue {
    value: AtomicI32,
    sync: Mutex<()>,
}

impl AtomicValue {
    pub fn new(value: i32) -> Self {
        Self {
            value: AtomicI32::new(value),
            sync: Mutex::new(()),
        }
    }

    pub fn store(&self, value: i32) {
        let _lock = self.sync.lock().unwrap();
        self.value.store(value, Ordering::Relaxed);
    }

    pub fn load(&self) -> i32 {
        let _lock = self.sync.lock().unwrap();
        self.value.load(Ordering::Relaxed)
    }
}
