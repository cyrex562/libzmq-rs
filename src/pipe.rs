#![allow(dead_code)]

use std::sync::Arc;
use std::collections::VecDeque;

// Message type placeholder
#[derive(Clone)]
pub struct Msg {
    data: Vec<u8>,
    flags: u32,
}

impl Msg {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            flags: 0,
        }
    }

    pub fn init_size(&mut self, size: usize) -> i32 {
        self.data.resize(size, 0);
        0
    }

    pub fn init_buffer(&mut self, data: &[u8], size: usize) -> i32 {
        self.data = data[..size].to_vec();
        0
    }

    pub fn close(&mut self) {
        self.data.clear();
    }

    pub fn is_delimiter(&self) -> bool {
        (self.flags & MsgFlags::DELIMITER as u32) != 0
    }

    pub fn is_routing_id(&self) -> bool {
        (self.flags & MsgFlags::ROUTING_ID as u32) != 0
    }

    pub fn is_credential(&self) -> bool {
        (self.flags & MsgFlags::CREDENTIAL as u32) != 0
    }

    pub fn flags(&self) -> u32 {
        self.flags
    }

    pub fn set_flags(&mut self, flags: u32) {
        self.flags = flags;
    }

    pub fn init_delimiter(&mut self) {
        self.flags = MsgFlags::DELIMITER as u32;
    }
}

#[derive(Clone, Copy)]
pub enum MsgFlags {
    MORE = 1,
    DELIMITER = 2,
    ROUTING_ID = 4,
    CREDENTIAL = 8,
}

// Basic pipe traits and types
pub trait PipeEvents {
    fn read_activated(&mut self, pipe: &mut Pipe);
    fn write_activated(&mut self, pipe: &mut Pipe);
    fn hiccuped(&mut self, pipe: &mut Pipe);
    fn pipe_terminated(&mut self, pipe: &mut Pipe);
}

#[derive(Clone)]
pub struct Blob {
    data: Vec<u8>,
}

impl Blob {
    pub fn set_deep_copy(&mut self, other: &Blob) {
        self.data = other.data.clone();
    }
}

#[derive(Clone)]
pub struct EndpointUriPair {
    local: String,
    remote: String,
}

// Basic pipe implementation
pub struct YPipe {
    queue: VecDeque<Msg>,
    conflate: bool,
}

impl YPipe {
    pub fn new(conflate: bool) -> Self {
        Self {
            queue: VecDeque::new(),
            conflate,
        }
    }

    pub fn write(&mut self, msg: Msg, more: bool) {
        if self.conflate && !more {
            self.queue.clear();
        }
        self.queue.push_back(msg);
    }

    pub fn read(&mut self, msg: &mut Msg) -> bool {
        if let Some(m) = self.queue.pop_front() {
            *msg = m;
            true
        } else {
            false
        }
    }

    pub fn check_read(&self) -> bool {
        !self.queue.is_empty()
    }

    pub fn probe<F>(&self, predicate: F) -> bool 
    where F: Fn(&Msg) -> bool 
    {
        self.queue.front().map_or(false, predicate)
    }

    pub fn flush(&mut self) -> bool {
        !self.queue.is_empty()
    }
}

pub struct Pipe {
    in_pipe: Option<YPipe>,
    out_pipe: Option<YPipe>,
    in_active: bool,
    out_active: bool,
    hwm: i32,
    lwm: i32,
    msgs_read: u64,
    msgs_written: u64,
    peers_msgs_read: u64,
    peer: Option<Arc<Pipe>>,
    sink: Option<Box<dyn PipeEvents>>,
    state: PipeState,
    delay: bool,
    router_socket_routing_id: Blob,
    server_socket_routing_id: u32,
    conflate: bool,
    endpoint_pair: EndpointUriPair,
    disconnect_msg: Msg,
}

#[derive(PartialEq)]
enum PipeState {
    Active,
    DelimiterReceived,
    WaitingForDelimiter,
    TermAckSent,
    TermReqSent1,
    TermReqSent2,
}

impl Pipe {
    pub fn new(
        in_pipe: YPipe,
        out_pipe: YPipe,
        inhwm: i32,
        outhwm: i32,
        conflate: bool,
    ) -> Self {
        Self {
            in_pipe: Some(in_pipe),
            out_pipe: Some(out_pipe),
            in_active: true,
            out_active: true,
            hwm: outhwm,
            lwm: Self::compute_lwm(inhwm),
            msgs_read: 0,
            msgs_written: 0,
            peers_msgs_read: 0,
            peer: None,
            sink: None,
            state: PipeState::Active,
            delay: true,
            router_socket_routing_id: Blob { data: Vec::new() },
            server_socket_routing_id: 0,
            conflate,
            endpoint_pair: EndpointUriPair {
                local: String::new(),
                remote: String::new(),
            },
            disconnect_msg: Msg::new(),
        }
    }

    pub fn set_peer(&mut self, peer: Arc<Pipe>) {
        self.peer = Some(peer);
    }

    pub fn check_read(&mut self) -> bool {
        if !self.in_active {
            return false;
        }
        if self.state != PipeState::Active && self.state != PipeState::WaitingForDelimiter {
            return false;
        }

        if let Some(in_pipe) = &mut self.in_pipe {
            if !in_pipe.check_read() {
                self.in_active = false;
                return false;
            }

            if in_pipe.probe(Msg::is_delimiter) {
                let mut msg = Msg::new();
                in_pipe.read(&mut msg);
                self.process_delimiter();
                return false;
            }
        }

        true
    }

    fn compute_lwm(hwm: i32) -> i32 {
        (hwm + 1) / 2
    }

    fn process_delimiter(&mut self) {
        match self.state {
            PipeState::Active => {
                self.state = PipeState::DelimiterReceived;
            }
            PipeState::WaitingForDelimiter => {
                self.rollback();
                self.out_pipe = None;
                // send_pipe_term_ack would go here
                self.state = PipeState::TermAckSent;
            }
            _ => {}
        }
    }

    fn rollback(&mut self) {
        if let Some(out_pipe) = &mut self.out_pipe {
            while let Some(msg) = out_pipe.queue.pop_back() {
                if msg.flags() & MsgFlags::MORE as u32 != 0 {
                    // Close message
                }
            }
        }
    }
}

// Create a pair of pipes for bidirectional communication
pub fn create_pipe_pair(
    conflate: [bool; 2],
    hwms: [i32; 2],
) -> (Pipe, Pipe) {
    let pipe1 = YPipe::new(conflate[0]);
    let pipe2 = YPipe::new(conflate[1]);

    let mut pipe_a = Pipe::new(
        pipe1,
        pipe2.clone(),
        hwms[0],
        hwms[1],
        conflate[0],
    );

    let mut pipe_b = Pipe::new(
        pipe2,
        pipe1.clone(),
        hwms[1],
        hwms[0],
        conflate[1],
    );

    let pipe_a_arc = Arc::new(pipe_a.clone());
    let pipe_b_arc = Arc::new(pipe_b.clone());

    pipe_a.set_peer(pipe_b_arc);
    pipe_b.set_peer(pipe_a_arc);

    (pipe_a, pipe_b)
}
