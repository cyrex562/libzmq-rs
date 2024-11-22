use crate::{
    constants::ZMQ_PUSH, context::Context, load_balancer::LoadBalancer, message::Message, pipe::Pipe, socket::SocketBase, utils::NonCopyable
};

/// Push socket type (ZMQ_PUSH)
pub struct Push {
    socket: SocketBase,
    lb: LoadBalancer,
}

impl Push {
    pub fn new(parent: &mut Context, tid: u32, sid: i32) -> Self {
        let mut socket = SocketBase::new(parent, tid, sid);
        socket.set_socket_type(ZMQ_PUSH);

        Self {
            socket,
            lb: LoadBalancer::new(),
        }
    }

    pub fn attach_pipe(
        &mut self,
        pipe: &mut Pipe,
        _subscribe_to_all: bool,
        _locally_initiated: bool,
    ) {
        // Don't delay pipe termination as there is no one
        // to receive the delimiter
        pipe.set_nodelay();

        debug_assert!(pipe.is_some());
        self.lb.attach(pipe);
    }

    pub fn send(&mut self, msg: &mut Message) -> i32 {
        self.lb.send(msg)
    }

    pub fn has_out(&self) -> bool {
        self.lb.has_out()
    }

    pub fn write_activated(&mut self, pipe: &mut Pipe) {
        self.lb.activated(pipe);
    }

    pub fn pipe_terminated(&mut self, pipe: &mut Pipe) {
        self.lb.pipe_terminated(pipe);
    }
}

// Implement NonCopyable trait
impl NonCopyable for Push {}
