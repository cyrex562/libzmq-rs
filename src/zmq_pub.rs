use std::os::raw::c_int;
use crate::context::Context;
use crate::message::Message;
use crate::pipe::Pipe;
use crate::xpub::XPub;
// Forward declarations
// pub struct Ctx;
// pub struct IoThread;
// pub struct Pipe;
// pub struct Msg;
// pub struct XPub;

// Equivalent to the C++ class
pub struct Pub {
    inner: XPub,
}

impl Pub {
    pub fn new(parent: &mut Context, tid: u32, sid: i32) -> Self {
        Pub {
            inner: XPub::new(parent, tid, sid)
        }
    }

    pub fn attach_pipe(&mut self, pipe: &mut Pipe, subscribe_to_all: bool, locally_initiated: bool) {
        // Assert pipe is valid
        debug_assert!(!pipe.is_null());

        // Don't delay pipe termination as there is no one
        // to receive the delimiter
        pipe.set_nodelay();

        self.inner.attach_pipe(pipe, subscribe_to_all, locally_initiated);
    }

    pub fn recv(&mut self, _msg: &mut Message) -> c_int {
        // Messages cannot be received from PUB socket
        // FIXME: raise ENOTSUP
        // errno::set_errno(errno::Errno(libc::ENOTSUP));
        -1
    }

    pub fn has_in(&self) -> bool {
        false
    }
}

// Prevent automatic copy and move implementations
// impl !Send for Pub {}
// impl !Sync for Pub {}
