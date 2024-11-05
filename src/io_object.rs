// SPDX-License-Identifier: MPL-2.0

use std::ptr::null_mut;

mod precompiled;
mod io_object;
mod io_thread;
mod err;

pub struct IoObject<'a> {
    poller: *mut Poller,
    io_thread: Option<&'a IoThread>,
}

impl<'a> IoObject<'a> {
    pub fn new(io_thread: Option<&'a IoThread>) -> IoObject<'a> {
        let mut obj = IoObject {
            poller: null_mut(),
            io_thread,
        };
        if let Some(thread) = io_thread {
            obj.plug(thread);
        }
        obj
    }

    pub fn plug(&mut self, io_thread: &'a IoThread) {
        assert!(!self.poller.is_null());
        self.poller = io_thread.get_poller();
    }

    pub fn unplug(&mut self) {
        assert!(!self.poller.is_null());
        self.poller = null_mut();
    }

    pub fn add_fd(&mut self, fd: Fd) -> Handle {
        unsafe { (*self.poller).add_fd(fd, self) }
    }

    pub fn rm_fd(&mut self, handle: Handle) {
        unsafe { (*self.poller).rm_fd(handle) }
    }

    pub fn set_pollin(&mut self, handle: Handle) {
        unsafe { (*self.poller).set_pollin(handle) }
    }

    pub fn reset_pollin(&mut self, handle: Handle) {
        unsafe { (*self.poller).reset_pollin(handle) }
    }

    pub fn set_pollout(&mut self, handle: Handle) {
        unsafe { (*self.poller).set_pollout(handle) }
    }

    pub fn reset_pollout(&mut self, handle: Handle) {
        unsafe { (*self.poller).reset_pollout(handle) }
    }

    pub fn add_timer(&mut self, timeout: i32, id: i32) {
        unsafe { (*self.poller).add_timer(timeout, self, id) }
    }

    pub fn cancel_timer(&mut self, id: i32) {
        unsafe { (*self.poller).cancel_timer(self, id) }
    }

    pub fn in_event(&self) {
        assert!(false);
    }

    pub fn out_event(&self) {
        assert!(false);
    }

    pub fn timer_event(&self, _: i32) {
        assert!(false);
    }
}

impl<'a> Drop for IoObject<'a> {
    fn drop(&mut self) {
        // Destructor logic if needed
    }
}

// Placeholder types for the example
type Poller = ();
type IoThread = ();
type Fd = i32;
type Handle = i32;