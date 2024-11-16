// SPDX-License-Identifier: MPL-2.0

use crate::blob::Blob;
use crate::context::Context;
use crate::io_thread::IoThread;
use crate::message::Message;
use crate::pipe::MsgFlags::MORE;
use crate::pipe::Pipe;
use crate::session_base::{SessionBase, ZMQ_CHANNEL};
use crate::socket_base::SocketBase;
use std::sync::Arc;

pub struct Channel {
    pipe: Option<Arc<dyn Pipe>>,
    // other fields from SocketBase
    socket_base: SocketBase,
}

impl Channel {
    pub fn new(parent: &Arc<Context>, tid: u32, sid: i32) -> Self {
        let mut channel = Channel {
            pipe: None,
            // initialize other fields from SocketBase
            socket_base: SocketBase::new(parent, tid, sid, false),
        };
        channel.socket_base.options.type_ = ZMQ_CHANNEL;
        channel
    }

    pub fn xattach_pipe(
        &mut self,
        pipe: Arc<Pipe>,
        subscribe_to_all: bool,
        locally_initiated: bool,
    ) {
        // ZMQ_PAIR socket can only be connected to a single peer.
        // The socket rejects any further connection requests.
        if self.pipe.is_none() {
            self.pipe = Some(pipe);
        } else {
            pipe.terminate(false);
        }
    }

    pub fn xpipe_terminated(&mut self, pipe: &Arc<Pipe>) {
        if self.pipe.as_ref() == Some(pipe) {
            self.pipe = None;
        }
    }

    pub fn xread_activated(&self, _pipe: &Arc<Pipe>) {
        // There's just one pipe. No lists of active and inactive pipes.
        // There's nothing to do here.
    }

    pub fn xwrite_activated(&self, _pipe: &Arc<Pipe>) {
        // There's just one pipe. No lists of active and inactive pipes.
        // There's nothing to do here.
    }

    pub fn xsend(&mut self, msg: &mut Message) -> Result<(), i32> {
        // CHANNEL sockets do not allow multipart data (ZMQ_SNDMORE)
        if msg.flags() & MORE != 0 {
            return Err(libc::EINVAL);
        }

        if let Some(pipe) = &self.pipe {
            if !pipe.write(msg) {
                return Err(libc::EAGAIN);
            }
            pipe.flush();
            msg.init().map_err(|_| libc::EINVAL)?;
            Ok(())
        } else {
            Err(libc::EAGAIN)
        }
    }

    pub fn xrecv(&mut self, msg: &mut Message) -> Result<(), i32> {
        msg.close().map_err(|_| libc::EINVAL)?;

        if let Some(pipe) = &self.pipe {
            let mut read = pipe.read(msg);
            while read && msg.flags() & MORE != 0 {
                read = pipe.read(msg);
                while read && msg.flags() & MORE != 0 {
                    read = pipe.read(msg);
                }
                if read {
                    read = pipe.read(msg);
                }
            }

            if !read {
                msg.init().map_err(|_| libc::EINVAL)?;
                return Err(libc::EAGAIN);
            }
            Ok(())
        } else {
            msg.init().map_err(|_| libc::EINVAL)?;
            Err(libc::EAGAIN)
        }
    }

    pub fn xhas_in(&self) -> bool {
        if let Some(pipe) = &self.pipe {
            pipe.check_read()
        } else {
            false
        }
    }

    pub fn xhas_out(&self) -> bool {
        if let Some(pipe) = &self.pipe {
            pipe.check_write()
        } else {
            false
        }
    }
}
