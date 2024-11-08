use std::sync::Mutex;
use crate::msg::Msg;
// Message type similar to the C++ msg_t
// #[derive(Clone)]
// pub struct Message {
//     // Implementation details omitted for brevity
// }

// impl Message {
//     fn new() -> Self {
//         Message { /* initialization */ }
//     }
// 
//     fn check(&self) -> bool {
//         true // Implement actual checking logic
//     }
// }

// Generic double buffer implementation
pub struct DBuffer<T> {
    storage: [T; 2],
    back_idx: usize,
    front_idx: usize,
    sync: Mutex<()>,
    has_msg: Mutex<bool>,
}

impl<T: Clone> DBuffer<T> {
    pub fn new(init_value: T) -> Self {
        DBuffer {
            storage: [init_value.clone(), init_value],
            back_idx: 0,
            front_idx: 1,
            sync: Mutex::new(()),
            has_msg: Mutex::new(false),
        }
    }

    pub fn write(&mut self, value: T) {
        self.storage[self.back_idx] = value;
        
        if let Ok(_guard) = self.sync.try_lock() {
            // Swap front and back
            std::mem::swap(&mut self.storage[self.front_idx], &mut self.storage[self.back_idx]);
            *self.has_msg.lock().unwrap() = true;
        }
    }

    pub fn read(&mut self) -> Option<T> {
        let _guard = self.sync.lock().unwrap();
        let has_msg = *self.has_msg.lock().unwrap();
        
        if !has_msg {
            return None;
        }

        let value = self.storage[self.front_idx].clone();
        *self.has_msg.lock().unwrap() = false;
        Some(value)
    }

    pub fn check_read(&self) -> bool {
        *self.has_msg.lock().unwrap()
    }

    pub fn probe<F>(&self, f: F) -> bool 
    where
        F: Fn(&T) -> bool
    {
        let _guard = self.sync.lock().unwrap();
        f(&self.storage[self.front_idx])
    }
}

// Specific implementation for Message type
impl DBuffer<Msg> {
    pub fn new_message_buffer() -> Self {
        Self::new(Msg::new())
    }

    pub fn write_message(&mut self, msg: Msg) {
        assert!(msg.check());
        self.write(msg);
    }
}

// Implement Drop if necessary
impl<T> Drop for DBuffer<T> {
    fn drop(&mut self) {
        // Cleanup resources if needed
    }
}