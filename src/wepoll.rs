#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::sync::Arc;
use windows_sys::Win32::{
    Foundation::*,
    Networking::WinSock::*,
    System::IO::*,
};

// Constants from original wepoll.h
pub const EPOLLIN: u32 = 1 << 0;
pub const EPOLLPRI: u32 = 1 << 1;
pub const EPOLLOUT: u32 = 1 << 2;
pub const EPOLLERR: u32 = 1 << 3;
pub const EPOLLHUP: u32 = 1 << 4;
pub const EPOLLRDNORM: u32 = 1 << 6;
pub const EPOLLRDBAND: u32 = 1 << 7;
pub const EPOLLWRNORM: u32 = 1 << 8;
pub const EPOLLWRBAND: u32 = 1 << 9;
pub const EPOLLMSG: u32 = 1 << 10;
pub const EPOLLRDHUP: u32 = 1 << 13;
pub const EPOLLONESHOT: u32 = 1 << 31;

pub const EPOLL_CTL_ADD: i32 = 1;
pub const EPOLL_CTL_MOD: i32 = 2;
pub const EPOLL_CTL_DEL: i32 = 3;

#[repr(C)]
#[derive(Clone, Copy)]
pub union epoll_data_t {
    pub ptr: *mut std::ffi::c_void,
    pub fd: i32,
    pub u32_: u32,
    pub u64_: u64,
    pub sock: SOCKET,
    pub hnd: HANDLE,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct epoll_event {
    pub events: u32,
    pub data: epoll_data_t,
}

pub struct EPoll {
    handle: HANDLE,
}

impl EPoll {
    pub fn new() -> windows::core::Result<Self> {
        unsafe {
            // Initialize Windows sockets if needed
            let mut wsadata = std::mem::zeroed();
            if WSAStartup(0x202, &mut wsadata) != 0 {
                return Err(windows::core::Error::from_win32());
            }

            let handle = epoll_create1(0);
            if handle.is_null() {
                return Err(windows::core::Error::from_win32());
            }

            Ok(EPoll { handle })
        }
    }

    pub fn add(&self, socket: SOCKET, event: &epoll_event) -> windows::core::Result<()> {
        unsafe {
            if epoll_ctl(self.handle, EPOLL_CTL_ADD as i32, socket, event as *const _) != 0 {
                return Err(windows::core::Error::from_win32());
            }
            Ok(())
        }
    }

    pub fn modify(&self, socket: SOCKET, event: &epoll_event) -> windows::core::Result<()> {
        unsafe {
            if epoll_ctl(self.handle, EPOLL_CTL_MOD as i32, socket, event as *const _) != 0 {
                return Err(windows::core::Error::from_win32());
            }
            Ok(())
        }
    }

    pub fn delete(&self, socket: SOCKET) -> windows::core::Result<()> {
        unsafe {
            if epoll_ctl(self.handle, EPOLL_CTL_DEL as i32, socket, std::ptr::null()) != 0 {
                return Err(windows::core::Error::from_win32());
            }
            Ok(())
        }
    }

    pub fn wait(&self, events: &mut [epoll_event], timeout: i32) -> windows::core::Result<usize> {
        unsafe {
            let res = epoll_wait(
                self.handle,
                events.as_mut_ptr(),
                events.len() as i32,
                timeout,
            );
            if res < 0 {
                return Err(windows::core::Error::from_win32());
            }
            Ok(res as usize)
        }
    }
}

impl Drop for EPoll {
    fn drop(&mut self) {
        unsafe {
            epoll_close(self.handle);
        }
    }
}

// FFI declarations for the wepoll C functions
extern "C" {
    fn epoll_create1(flags: i32) -> HANDLE;
    fn epoll_close(ephnd: HANDLE) -> i32;
    fn epoll_ctl(ephnd: HANDLE, op: i32, sock: SOCKET, event: *const epoll_event) -> i32;
    fn epoll_wait(ephnd: HANDLE, events: *mut epoll_event, maxevents: i32, timeout: i32) -> i32;
}

// Safe wrapper functions
pub fn create_epoll() -> windows::core::Result<EPoll> {
    EPoll::new()
}
