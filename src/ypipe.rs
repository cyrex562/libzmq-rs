use std::sync::atomic::{AtomicPtr, Ordering};
use std::ptr;
use std::marker::PhantomData;

const N: usize = 256; // Default granularity, can be adjusted

pub struct YQueue<T> {
    chunks: Vec<Vec<T>>,
    begin_pos: usize,
    end_pos: usize,
    back_chunk: usize,
    front_chunk: usize,
    spare_chunk: Option<Vec<T>>,
}

impl<T> YQueue<T> {
    pub fn new() -> Self {
        let mut chunks = Vec::new();
        chunks.push(Vec::with_capacity(N));
        YQueue {
            chunks,
            begin_pos: 0,
            end_pos: 0,
            back_chunk: 0,
            front_chunk: 0,
            spare_chunk: None,
        }
    }

    pub fn push(&mut self) {
        if self.end_pos == N {
            if self.spare_chunk.is_some() {
                self.chunks.push(self.spare_chunk.take().unwrap());
            } else {
                self.chunks.push(Vec::with_capacity(N));
            }
            self.back_chunk += 1;
            self.end_pos = 0;
        }
        self.chunks[self.back_chunk].push(unsafe { std::mem::zeroed() });
        self.end_pos += 1;
    }

    pub fn pop(&mut self) {
        self.begin_pos += 1;
        if self.begin_pos == N {
            self.spare_chunk = Some(self.chunks.remove(0));
            self.begin_pos = 0;
            self.front_chunk += 1;
        }
    }
}

pub struct YPipe<T> {
    queue: YQueue<T>,
    w: *mut T,
    r: *mut T,
    f: *mut T,
    c: AtomicPtr<T>,
    _marker: PhantomData<T>,
}

impl<T> YPipe<T> {
    pub fn new() -> Self {
        let mut queue = YQueue::new();
        queue.push();
        let back_ptr = queue.chunks[0].as_mut_ptr();
        
        YPipe {
            queue,
            w: back_ptr,
            r: back_ptr,
            f: back_ptr,
            c: AtomicPtr::new(back_ptr),
            _marker: PhantomData,
        }
    }

    pub fn write(&mut self, value: T, incomplete: bool) {
        unsafe {
            ptr::write(self.w, value);
        }
        self.queue.push();
        self.w = self.queue.chunks[self.queue.back_chunk].as_mut_ptr();
        
        if !incomplete {
            self.f = self.w;
        }
    }

    pub fn flush(&mut self) -> bool {
        if self.w == self.f {
            return true;
        }

        let prev = self.c.compare_exchange(
            self.w,
            self.f,
            Ordering::SeqCst,
            Ordering::SeqCst
        );

        if prev.is_err() {
            self.c.store(self.f, Ordering::SeqCst);
            self.w = self.f;
            return false;
        }

        self.w = self.f;
        true
    }

    pub fn read(&mut self) -> Option<T> {
        if !self.check_read() {
            return None;
        }

        let value = unsafe {
            ptr::read(self.r)
        };
        self.queue.pop();
        self.r = if self.queue.begin_pos == 0 {
            self.queue.chunks[self.queue.front_chunk].as_mut_ptr()
        } else {
            unsafe { self.r.add(1) }
        };
        Some(value)
    }

    fn check_read(&mut self) -> bool {
        if self.r != self.queue.chunks[self.queue.front_chunk].as_mut_ptr() {
            return true;
        }

        let front_ptr = self.queue.chunks[self.queue.front_chunk].as_mut_ptr();
        let prev = self.c.compare_exchange(
            front_ptr,
            ptr::null_mut(),
            Ordering::SeqCst,
            Ordering::SeqCst
        );

        match prev {
            Ok(ptr) => {
                self.r = ptr;
                self.r != front_ptr
            }
            Err(_) => false
        }
    }
}

unsafe impl<T: Send> Send for YPipe<T> {}
unsafe impl<T: Send> Sync for YPipe<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let mut pipe = YPipe::new();
        pipe.write(42, false);
        assert!(pipe.flush());
        assert_eq!(pipe.read(), Some(42));
    }
}
