#![allow(dead_code)]

use std::mem;
use std::ptr;

#[cfg(windows)]
use winapi::um::winsock2::{fd_set, SOCKET};

// Fast vector implementation with static buffer optimization
pub struct FastVector<T, const S: usize> {
    static_buf: [T; S],
    buf: *mut T,
    len: usize,
}

impl<T: Default + Copy, const S: usize> FastVector<T, S> {
    pub fn new(nitems: usize) -> Self {
        let static_buf = [T::default(); S];
        let buf = if nitems > S {
            let v = vec![T::default(); nitems].into_boxed_slice();
            Box::leak(v).as_mut_ptr()
        } else {
            static_buf.as_ptr() as *mut T
        };

        Self {
            static_buf,
            buf,
            len: nitems,
        }
    }

    pub fn get(&self, i: usize) -> &T {
        unsafe { &*self.buf.add(i) }
    }

    pub fn get_mut(&mut self, i: usize) -> &mut T {
        unsafe { &mut *self.buf.add(i) }
    }
}

impl<T, const S: usize> Drop for FastVector<T, S> {
    fn drop(&mut self) {
        if self.buf != self.static_buf.as_ptr() as *mut T {
            unsafe {
                Vec::from_raw_parts(self.buf, self.len, self.len);
            }
        }
    }
}

// Resizable fast vector implementation
pub struct ResizableFastVector<T, const S: usize> {
    static_buf: [T; S],
    dynamic_buf: Option<Vec<T>>,
}

impl<T: Default + Copy, const S: usize> ResizableFastVector<T, S> {
    pub fn new() -> Self {
        Self {
            static_buf: [T::default(); S],
            dynamic_buf: None,
        }
    }

    pub fn resize(&mut self, nitems: usize) {
        if let Some(buf) = &mut self.dynamic_buf {
            buf.resize(nitems, T::default());
        } else if nitems > S {
            let mut new_buf = Vec::with_capacity(nitems);
            new_buf.extend_from_slice(&self.static_buf);
            new_buf.resize(nitems, T::default());
            self.dynamic_buf = Some(new_buf);
        }
    }

    pub fn get_buf(&mut self) -> &mut [T] {
        match &mut self.dynamic_buf {
            Some(buf) => buf,
            None => &mut self.static_buf,
        }
    }
}

#[cfg(feature = "poll")]
pub fn compute_timeout(first_pass: bool, timeout: i64, now: u64, end: u64) -> i32 {
    if first_pass {
        return 0;
    }

    if timeout < 0 {
        return -1;
    }

    std::cmp::min(end.saturating_sub(now), i32::MAX as u64) as i32
}

#[cfg(all(windows, feature = "select"))]
pub fn valid_pollset_bytes(pollset: &fd_set) -> usize {
    let fd_count = unsafe { (*pollset).fd_count as usize };
    mem::size_of::<SOCKET>() * (1 + fd_count)
}

#[cfg(all(not(windows), feature = "select"))]
pub fn valid_pollset_bytes(_pollset: &fd_set) -> usize {
    mem::size_of::<fd_set>()
}

#[cfg(windows)]
pub struct OptimizedFdSet {
    fd_set: FastVector<SOCKET, { 1 + 16 }>, // ZMQ_POLLITEMS_DFLT = 16
}

#[cfg(windows)]
impl OptimizedFdSet {
    pub fn new(nevents: usize) -> Self {
        Self {
            fd_set: FastVector::new(1 + nevents),
        }
    }

    pub fn get(&mut self) -> *mut fd_set {
        self.fd_set.buf as *mut fd_set
    }
}

#[cfg(not(windows))]
pub struct OptimizedFdSet {
    fd_set: fd_set,
}

#[cfg(not(windows))]
impl OptimizedFdSet {
    pub fn new(_nevents: usize) -> Self {
        Self {
            fd_set: unsafe { mem::zeroed() },
        }
    }

    pub fn get(&mut self) -> *mut fd_set {
        &mut self.fd_set
    }
}
