use crate::{
    context::Context,
    fair_queue::FairQueue,
    message::Message,
    pipe::Pipe,
    socket::{Socket, SocketBase},
};

/// PULL socket implementation
pub struct Pull {
    /// Fair queuing object for inbound pipes
    fair_queue: FairQueue,
    /// Base socket functionality
    socket: SocketBase,
}

impl Pull {
    pub fn new(parent: &Context, tid: u32, sid: i32) -> Self {
        Self {
            fair_queue: FairQueue::new(),
            socket: SocketBase::new(parent, tid, sid, ZMQ_PULL),
        }
    }
}

impl Socket for Pull {
    fn attach_pipe(&mut self, pipe: &mut Pipe, _subscribe_to_all: bool, _locally_initiated: bool) {
        debug_assert!(pipe.is_some());
        self.fair_queue.attach(pipe);
    }

    fn recv(&mut self, msg: &mut Message) -> Result<(), i32> {
        self.fair_queue.recv(msg)
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
}

#[cfg(test)]
mod tests {
    use super::*;
    // Add tests here
}
