use std::sync::atomic::{AtomicPtr, Ordering};
use std::alloc::{Layout, alloc, dealloc};
use std::ptr;
use std::marker::PhantomData;

const CACHE_LINE_SIZE: usize = 64; // Equivalent to ZMQ_CACHELINE_SIZE

#[repr(C, align(64))] // Align to cache line size
struct Chunk<T, const N: usize> {
    values: [T; N],
    prev: *mut Chunk<T, N>,
    next: *mut Chunk<T, N>,
}

pub struct YQueue<T, const N: usize> {
    begin_chunk: *mut Chunk<T, N>,
    begin_pos: usize,
    back_chunk: *mut Chunk<T, N>,
    back_pos: usize,
    end_chunk: *mut Chunk<T, N>,
    end_pos: usize,
    spare_chunk: AtomicPtr<Chunk<T, N>>,
    _marker: PhantomData<T>,
}

impl<T, const N: usize> YQueue<T, N> {
    pub fn new() -> Self {
        unsafe {
            let chunk = Self::allocate_chunk();
            YQueue {
                begin_chunk: chunk,
                begin_pos: 0,
                back_chunk: ptr::null_mut(),
                back_pos: 0,
                end_chunk: chunk,
                end_pos: 0,
                spare_chunk: AtomicPtr::new(ptr::null_mut()),
                _marker: PhantomData,
            }
        }
    }

    pub fn front(&self) -> &T {
        unsafe {
            &(*self.begin_chunk).values[self.begin_pos]
        }
    }

    pub fn front_mut(&mut self) -> &mut T {
        unsafe {
            &mut (*self.begin_chunk).values[self.begin_pos]
        }
    }

    pub fn back(&self) -> &T {
        unsafe {
            &(*self.back_chunk).values[self.back_pos]
        }
    }

    pub fn back_mut(&mut self) -> &mut T {
        unsafe {
            &mut (*self.back_chunk).values[self.back_pos]
        }
    }

    pub fn push(&mut self) {
        unsafe {
            self.back_chunk = self.end_chunk;
            self.back_pos = self.end_pos;

            self.end_pos += 1;
            if self.end_pos != N {
                return;
            }

            let sc = self.spare_chunk.swap(ptr::null_mut(), Ordering::AcqRel);
            if !sc.is_null() {
                (*self.end_chunk).next = sc;
                (*sc).prev = self.end_chunk;
            } else {
                let new_chunk = Self::allocate_chunk();
                (*self.end_chunk).next = new_chunk;
                (*new_chunk).prev = self.end_chunk;
            }
            self.end_chunk = (*self.end_chunk).next;
            self.end_pos = 0;
        }
    }

    pub fn unpush(&mut self) {
        unsafe {
            if self.back_pos != 0 {
                self.back_pos -= 1;
            } else {
                self.back_pos = N - 1;
                self.back_chunk = (*self.back_chunk).prev;
            }

            if self.end_pos != 0 {
                self.end_pos -= 1;
            } else {
                self.end_pos = N - 1;
                self.end_chunk = (*self.end_chunk).prev;
                let next = (*self.end_chunk).next;
                Self::deallocate_chunk(next);
                (*self.end_chunk).next = ptr::null_mut();
            }
        }
    }

    pub fn pop(&mut self) {
        unsafe {
            self.begin_pos += 1;
            if self.begin_pos == N {
                let old_chunk = self.begin_chunk;
                self.begin_chunk = (*self.begin_chunk).next;
                (*self.begin_chunk).prev = ptr::null_mut();
                self.begin_pos = 0;

                let cs = self.spare_chunk.swap(old_chunk, Ordering::AcqRel);
                if !cs.is_null() {
                    Self::deallocate_chunk(cs);
                }
            }
        }
    }

    unsafe fn allocate_chunk() -> *mut Chunk<T, N> {
        let layout = Layout::new::<Chunk<T, N>>();
        let ptr = alloc(layout) as *mut Chunk<T, N>;
        (*ptr).prev = ptr::null_mut();
        (*ptr).next = ptr::null_mut();
        ptr
    }

    unsafe fn deallocate_chunk(chunk: *mut Chunk<T, N>) {
        let layout = Layout::new::<Chunk<T, N>>();
        dealloc(chunk as *mut u8, layout);
    }
}

impl<T, const N: usize> Drop for YQueue<T, N> {
    fn drop(&mut self) {
        unsafe {
            while !self.begin_chunk.is_null() {
                if self.begin_chunk == self.end_chunk {
                    Self::deallocate_chunk(self.begin_chunk);
                    break;
                }
                let old = self.begin_chunk;
                self.begin_chunk = (*self.begin_chunk).next;
                Self::deallocate_chunk(old);
            }

            let spare = self.spare_chunk.load(Ordering::Acquire);
            if !spare.is_null() {
                Self::deallocate_chunk(spare);
            }
        }
    }
}

unsafe impl<T: Send, const N: usize> Send for YQueue<T, N> {}
unsafe impl<T: Sync, const N: usize> Sync for YQueue<T, N> {}
