use crate::constants::ZMQ_PULL;
use crate::mailbox::Mailbox;
use crate::socket_base::SocketBehavior;
use crate::{
    context::Context, fair_queue::FairQueue, message::Message, pipe::Pipe, socket_base::SocketBase,
};

/// PULL socket implementation
pub struct Pull {
    /// Fair queuing object for inbound pipes
    fair_queue: FairQueue,
    /// Base socket functionality
    socket: SocketBase,

    pipe: Pipe,
}

impl Pull {
    pub fn new(parent: &Context, tid: u32, sid: i32) -> Self {
        Self {
            fair_queue: FairQueue::new(),
            socket: SocketBase::new(parent, tid, sid, ZMQ_PULL),
            pipe: Pipe::new(),
        }
    }
}

impl SocketBehavior for Pull {
    fn attach_pipe(&mut self, pipe: &mut Pipe, _subscribe_to_all: bool, _locally_initiated: bool) {
        debug_assert!(pipe.is_some());
        self.fair_queue.attach(pipe);
    }

    fn recv(&mut self, msg: &mut Message) -> Result<(), i32> {
        *msg = self.fair_queue.recv();
        Ok(())
    }

    fn has_in(&self) -> bool {
        self.fair_queue.has_in()
    }

    fn read_activated(&mut self, pipe: &mut Pipe) {
        self.fair_queue.activated(pipe);
    }

    fn pipe_terminated(&mut self, pipe: &mut Pipe) {
        self.fair_queue.pipe_terminated(pipe);
    }

    fn check_tag(&self) -> bool {
        todo!()
    }

    fn is_thread_safe(&self) -> bool {
        todo!()
    }

    fn get_mailbox(&self) -> Option<&Mailbox> {
        todo!()
    }

    fn bind(&mut self, endpoint: &str) -> Result<()> {
        todo!()
    }

    fn connect(&mut self, endpoint: &str) -> Result<()> {
        todo!()
    }

    fn send(&mut self, msg: Message, flags: i32) -> Result<()> {
        todo!()
    }

    fn close(&mut self) -> Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    // Add tests here
}
