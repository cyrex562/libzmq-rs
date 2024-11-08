/* SPDX-License-Identifier: MPL-2.0 */
use libc::EINVAL;
use crate::ctx::Ctx;
use crate::precompiled::*;
use crate::macros::*;
use crate::err::*;
use crate::fq::FairQueue;
use crate::lb::Lb;
use crate::msg::*;
use crate::options::Options;
use crate::pipe::Pipe;
use crate::session_base::ZMQ_CLIENT;
use crate::socket_base::SocketBase;

pub struct Client<T: Pipe> {
    options: Options,
    fq: FairQueue<T: Pipe>,
    lb: Lb,
    socket_base: SocketBase
}

impl Client<T> {
    pub fn new(parent: &Ctx, tid: u32, sid: i32) -> Self {
        let mut options = Options::default();
        options.can_send_hello_msg = true;
        options.can_recv_hiccup_msg = true;

        let mut c = Client {
            options,
            fq: FairQueue::new(),
            lb: Lb::new(),
            socket_base: SocketBase::new(parent, tid, sid, false)
        };
        c.socket_base.options.type_ = ZMQ_CLIENT;
        
    }

    pub fn attach_pipe(&mut self, pipe: &Pipe, subscribe_to_all: bool, locally_initiated: bool) {
        // LIBZMQ_UNUSED!(subscribe_to_all);
        // LIBZMQ_UNUSED!(locally_initiated);

        // zmq_assert!(pipe);

        self.fq.attach(pipe);
        self.lb.attach(pipe);
    }

    pub fn send(&mut self, msg: &mut Msg) -> Result<(), i32> {
        // CLIENT sockets do not allow multipart data (ZMQ_SNDMORE)
        if msg.flags() & MsgFlags::More != 0 {
            return Err(EINVAL);
        }
        self.lb.sendpipe(msg, None)
    }

    pub fn recv(&mut self, msg: &mut Msg) -> Result<(), i32> {
        let mut rc = self.fq.recvpipe(msg, None);

        // Drop any messages with more flag
        while rc.is_ok() && msg.flags() & MsgFlags::More != 0 {
            // drop all frames of the current multi-frame message
            rc = self.fq.recvpipe(msg, None);

            while rc.is_ok() && msg.flags() & MsgFlags::More != 0 {
                rc = self.fq.recvpipe(msg, None);
            }

            // get the new message
            if rc.is_ok() {
                rc = self.fq.recvpipe(msg, None);
            }
        }

        rc
    }

    pub fn has_in(&mut self) -> bool {
        self.fq.has_in()
    }

    pub fn has_out(&mut self) -> bool {
        self.lb.has_out()
    }

    pub fn read_activated(&mut self, pipe: &Pipe) {
        self.fq.activated(pipe);
    }

    pub fn write_activated(&mut self, pipe: &Pipe) {
        self.lb.activated(pipe);
    }

    pub fn pipe_terminated(&mut self, pipe: &Pipe) {
        self.fq.pipe_terminated(pipe);
        self.lb.pipe_terminated(pipe);
    }
}
