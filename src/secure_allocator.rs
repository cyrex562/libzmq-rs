#![allow(unused_imports)]

use std::alloc::{GlobalAlloc, Layout};
use std::ptr::NonNull;
use std::marker::PhantomData;

// Feature flag for libsodium support
#[cfg(feature = "libsodium")]
extern "C" {
    fn sodium_allocarray(count: usize, size: usize) -> *mut u8;
    fn sodium_free(ptr: *mut u8);
}

#[cfg(feature = "libsodium")]
pub struct SecureAllocator<T> {
    _phantom: PhantomData<T>,
}

#[cfg(feature = "libsodium")]
impl<T> SecureAllocator<T> {
    pub const fn new() -> Self {
        SecureAllocator {
            _phantom: PhantomData,
        }
    }

    pub fn allocate(&self, count: usize) -> Option<NonNull<T>> {
        unsafe {
            let ptr = sodium_allocarray(std::mem::size_of::<T>(), count);
            NonNull::new(ptr as *mut T)
        }
    }

    pub fn deallocate(&self, ptr: NonNull<T>) {
        unsafe {
            sodium_free(ptr.as_ptr() as *mut u8);
        }
    }
}

// Default implementation when libsodium is not available
#[cfg(not(feature = "libsodium"))]
pub type SecureAllocator<T> = std::alloc::System;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_allocator() {
        let allocator = SecureAllocator::<u8>::new();
        if let Some(ptr) = allocator.allocate(1) {
            allocator.deallocate(ptr);
        }
    }
}
