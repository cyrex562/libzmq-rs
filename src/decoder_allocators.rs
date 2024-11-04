use std::sync::atomic::{AtomicUsize, Ordering};
use std::alloc::{alloc, dealloc, Layout};
use std::ptr::NonNull;

const MSG_VSM_SIZE: usize = 32; // Assuming max_vsm_size from original code

// Single buffer allocator
pub struct SingleAllocator {
    buf_size: usize,
    buf: NonNull<u8>,
}

impl SingleAllocator {
    pub fn new(buf_size: usize) -> Self {
        let layout = Layout::array::<u8>(buf_size).unwrap();
        let buf = unsafe { NonNull::new(alloc(layout)).expect("allocation failed") };
        
        Self { buf_size, buf }
    }

    pub fn allocate(&self) -> *mut u8 {
        self.buf.as_ptr()
    }

    pub fn size(&self) -> usize {
        self.buf_size
    }
}

impl Drop for SingleAllocator {
    fn drop(&mut self) {
        unsafe {
            let layout = Layout::array::<u8>(self.buf_size).unwrap();
            dealloc(self.buf.as_ptr(), layout);
        }
    }
}

// Shared message memory allocator
pub struct SharedMessageMemoryAllocator {
    buf: Option<NonNull<u8>>,
    buf_size: usize,
    max_size: usize,
    msg_content: Option<NonNull<u8>>,
    max_counters: usize,
}

impl SharedMessageMemoryAllocator {
    pub fn new(bufsize: usize) -> Self {
        let max_counters = (bufsize + MSG_VSM_SIZE - 1) / MSG_VSM_SIZE;
        Self {
            buf: None,
            buf_size: 0,
            max_size: bufsize,
            msg_content: None,
            max_counters,
        }
    }

    pub fn with_max_messages(bufsize: usize, max_messages: usize) -> Self {
        Self {
            buf: None,
            buf_size: 0,
            max_size: bufsize,
            msg_content: None,
            max_counters: max_messages,
        }
    }

    pub fn allocate(&mut self) -> *mut u8 {
        if let Some(buf) = self.buf {
            let counter = unsafe { &*(buf.as_ptr() as *const AtomicUsize) };
            if counter.fetch_sub(1, Ordering::AcqRel) > 1 {
                self.release();
            }
        }

        if self.buf.is_none() {
            let alloc_size = self.max_size + std::mem::size_of::<AtomicUsize>() 
                          + self.max_counters * std::mem::size_of::<usize>();
            let layout = Layout::array::<u8>(alloc_size).unwrap();
            let new_buf = unsafe { NonNull::new(alloc(layout)).expect("allocation failed") };
            
            unsafe {
                // Initialize atomic counter
                let counter = new_buf.as_ptr() as *mut AtomicUsize;
                std::ptr::write(counter, AtomicUsize::new(1));
            }
            
            self.buf = Some(new_buf);
        } else {
            let counter = unsafe { &*(self.buf.unwrap().as_ptr() as *const AtomicUsize) };
            counter.store(1, Ordering::Release);
        }

        self.buf_size = self.max_size;
        let base_ptr = self.buf.unwrap().as_ptr();
        self.msg_content = NonNull::new(unsafe { 
            base_ptr.add(std::mem::size_of::<AtomicUsize>() + self.max_size)
        });

        unsafe { base_ptr.add(std::mem::size_of::<AtomicUsize>()) }
    }

    pub fn deallocate(&mut self) {
        if let Some(buf) = self.buf {
            let counter = unsafe { &*(buf.as_ptr() as *const AtomicUsize) };
            if counter.fetch_sub(1, Ordering::AcqRel) == 1 {
                let alloc_size = self.max_size + std::mem::size_of::<AtomicUsize>() 
                              + self.max_counters * std::mem::size_of::<usize>();
                let layout = Layout::array::<u8>(alloc_size).unwrap();
                unsafe { dealloc(buf.as_ptr(), layout); }
            }
        }
        self.clear();
    }

    pub fn inc_ref(&self) {
        if let Some(buf) = self.buf {
            let counter = unsafe { &*(buf.as_ptr() as *const AtomicUsize) };
            counter.fetch_add(1, Ordering::AcqRel);
        }
    }

    pub fn size(&self) -> usize {
        self.buf_size
    }

    pub fn data(&self) -> Option<*mut u8> {
        self.buf.map(|buf| unsafe { 
            buf.as_ptr().add(std::mem::size_of::<AtomicUsize>())
        })
    }

    fn clear(&mut self) {
        self.buf = None;
        self.buf_size = 0;
        self.msg_content = None;
    }

    fn release(&mut self) -> Option<NonNull<u8>> {
        let buf = self.buf;
        self.clear();
        buf
    }
}

impl Drop for SharedMessageMemoryAllocator {
    fn drop(&mut self) {
        self.deallocate();
    }
}
