use crate::{
    context::Context,
    fair_queue::FairQueue,
    load_balancer::LoadBalancer,
    message::Message,
    pipe::Pipe,
    socket::SocketBase,
    socket_options::{SocketOptions, ZMQ_DEALER},
};

pub struct Dealer {
    socket_base: SocketBase,
    fq: FairQueue,
    lb: LoadBalancer,
    probe_router: bool,
}

impl Dealer {
    pub fn new(parent: &Context, tid: u32, sid: i32) -> Self {
        let mut options = SocketOptions::default();
        options.socket_type = ZMQ_DEALER;
        options.can_send_hello_msg = true;
        options.can_recv_hiccup_msg = true;

        Dealer {
            socket_base: SocketBase::new(parent, tid, sid, options),
            fq: FairQueue::new(),
            lb: LoadBalancer::new(),
            probe_router: false,
        }
    }

    pub fn attach_pipe(&mut self, pipe: &mut Pipe, _subscribe_to_all: bool, _locally_initiated: bool) {
        if self.probe_router {
            let probe_msg = Message::new();
            if let Ok(msg) = probe_msg {
                let _ = pipe.write(&msg);
                pipe.flush();
            }
        }

        self.fq.attach(pipe);
        self.lb.attach(pipe);
    }

    pub fn set_sock_opt(&mut self, option: i32, optval: &[u8]) -> Result<(), i32> {
        if option == ZMQ_PROBE_ROUTER && optval.len() == std::mem::size_of::<i32>() {
            let value = i32::from_ne_bytes(optval.try_into().unwrap());
            if value >= 0 {
                self.probe_router = value != 0;
                return Ok(());
            }
        }
        Err(libc::EINVAL)
    }

    pub fn send(&mut self, msg: &mut Message) -> Result<(), i32> {
        self.sendpipe(msg, None)
    }

    pub fn recv(&mut self, msg: &mut Message) -> Result<(), i32> {
        self.recvpipe(msg, None)
    }

    pub fn has_in(&self) -> bool {
        self.fq.has_in()
    }

    pub fn has_out(&self) -> bool {
        self.lb.has_out()
    }

    pub fn read_activated(&mut self, pipe: &mut Pipe) {
        self.fq.activated(pipe);
    }

    pub fn write_activated(&mut self, pipe: &mut Pipe) {
        self.lb.activated(pipe);
    }

    pub fn pipe_terminated(&mut self, pipe: &mut Pipe) {
        self.fq.pipe_terminated(pipe);
        self.lb.pipe_terminated(pipe);
    }

    pub fn sendpipe(&mut self, msg: &mut Message, pipe: Option<&mut Pipe>) -> Result<(), i32> {
        self.lb.sendpipe(msg, pipe)
    }

    pub fn recvpipe(&mut self, msg: &mut Message, pipe: Option<&mut Pipe>) -> Result<(), i32> {
        self.fq.recvpipe(msg, pipe)
    }
}
