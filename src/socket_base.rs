#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use crate::constants::{ZMQ_BLOCKY, ZMQ_EPROTONOSUPPORT, ZMQ_ETERM, ZMQ_IPV6};
use crate::context::Context;
use crate::zmq_draft::ZMQ_ZERO_COPY_RECV;
use std::collections::HashMap;

// Type aliases
type ZmqResult<T> = Result<T, i32>; // Using i32 for errno compatibility

// Constants
const ZMQ_PAIR: i32 = 0;
const ZMQ_PUB: i32 = 1;
const ZMQ_SUB: i32 = 2;
const ZMQ_REQ: i32 = 3;
const ZMQ_REP: i32 = 4;
const ZMQ_DEALER: i32 = 5;
const ZMQ_ROUTER: i32 = 6;
// ...etc

// Core traits
pub trait SocketBehavior {
    fn check_tag(&self) -> bool;
    fn is_thread_safe(&self) -> bool;
    fn get_mailbox(&self) -> Option<&Mailbox>;
    fn bind(&mut self, endpoint: &str) -> ZmqResult<()>;
    fn connect(&mut self, endpoint: &str) -> ZmqResult<()>;
    fn send(&mut self, msg: Message, flags: i32) -> ZmqResult<()>;
    fn recv(&mut self, flags: i32) -> ZmqResult<Message>;
    fn close(&mut self) -> ZmqResult<()>;

    fn attach_pipe(&mut self, pipe: &mut Pipe, _subscribe_to_all: bool, _locally_initiated: bool);

    fn has_in(&self) -> bool;

    fn read_activated(&mut self, pipe: &mut Pipe);

    fn pipe_terminated(&mut self, pipe: &mut Pipe);
}

// Core structures
#[derive(Default)]
pub struct SocketOptions {
    pub socket_id: i32,
    pub type_: i32,
    pub immediate: i32,
    pub rcvmore: bool,
    pub sndhwm: i32,
    pub rcvhwm: i32,
    // ... etc
}

pub struct SocketBase {
    pub options: SocketOptions,
    pub mailbox: Option<Mailbox>,
    pub pipes: Vec<Pipe>,
    pub endpoints: HashMap<String, Endpoint>,
    pub monitor_socket: Option<Box<dyn SocketBehavior>>,
    pub monitor_events: u64,
    thread_safe: bool,
    tag: u32,
    ctx_terminated: bool,
    destroyed: bool,
    disconnected: bool,
}

impl SocketBase {
    pub fn new(ctx: &Context, tid: u32, sid: i32, thread_safe: bool) -> Self {
        let mut socket = SocketBase {
            options: SocketOptions::default(),
            mailbox: None,
            pipes: Vec::new(),
            endpoints: HashMap::new(),
            monitor_socket: None,
            monitor_events: 0,
            thread_safe: thread_safe,
            tag: 0xbaddecaf,
            ctx_terminated: false,
            destroyed: false,
            disconnected: false,
        };

        socket.options.socket_id = sid;
        // socket.options.type_ = ctx.get_option(ZMQ_SOCKET_TYPE);
        // options.socket_id = sid_;
        // options.ipv6 = (parent_->get (ZMQ_IPV6) != 0);
        socket.options.ipv6 = socket.parent_.get(ZMQ_IPV6) != 0;
        // options.linger.store (parent_->get (ZMQ_BLOCKY) ? -1 : 0);
        socket.options.linger = if socket.parent_.get(ZMQ_BLOCKY) {
            -1
        } else {
            0
        };
        // options.zero_copy = parent_->get (ZMQ_ZERO_COPY_RECV) != 0;
        socket.options.zero_copy = socket.parent_.get(ZMQ_ZERO_COPY_RECV) != 0;

        if thread_safe {
            socket.mailbox = Some(Mailbox::new_safe());
        } else {
            socket.mailbox = Some(Mailbox::new());
        }

        socket
    }

    pub fn create(
        socket_type: i32,
        ctx: &Context,
        tid: u32,
        sid: i32,
    ) -> Option<Box<dyn SocketBehavior>> {
        match socket_type {
            ZMQ_PAIR => Some(Box::new(PairSocket::new(ctx, tid, sid))),
            ZMQ_PUB => Some(Box::new(PubSocket::new(ctx, tid, sid))),
            ZMQ_SUB => Some(Box::new(SubSocket::new(ctx, tid, sid))),
            // ... etc
            _ => {
                // errno = EINVAL
                None
            }
        }
    }

    // Socket behavior implementation
    fn check_tag(&self) -> bool {
        self.tag == 0xbaddecaf
    }

    pub fn is_thread_safe(&self) -> bool {
        self.thread_safe
    }

    fn get_mailbox(&self) -> Option<&Mailbox> {
        self.mailbox.as_ref()
    }

    // Main socket operations
    fn bind(&mut self, endpoint: &str) -> ZmqResult<()> {
        if self.ctx_terminated {
            return Err(ZMQ_ETERM);
        }

        // Parse endpoint URI
        let (protocol, address) = self.parse_uri(endpoint)?;

        // Check protocol
        self.check_protocol(&protocol)?;

        match protocol.as_str() {
            "inproc" => self.bind_inproc(&address),
            "tcp" => self.bind_tcp(&address),
            // ... etc
            _ => Err(ZMQ_EPROTONOSUPPORT),
        }
    }

    fn connect(&mut self, endpoint: &str) -> ZmqResult<()> {
        if self.ctx_terminated {
            return Err(ZMQ_ETERM);
        }

        let (protocol, address) = self.parse_uri(endpoint)?;
        self.check_protocol(&protocol)?;

        match protocol.as_str() {
            "inproc" => self.connect_inproc(&address),
            "tcp" => self.connect_tcp(&address),
            // ... etc
            _ => Err(libc::EPROTONOSUPPORT),
        }
    }

    // Helper functions
    fn parse_uri(&self, uri: &str) -> ZmqResult<(String, String)> {
        if let Some(idx) = uri.find("://") {
            let (protocol, address) = uri.split_at(idx);
            let address = &address[3..]; // Skip "://"

            if !protocol.is_empty() && !address.is_empty() {
                Ok((protocol.to_string(), address.to_string()))
            } else {
                Err(libc::EINVAL)
            }
        } else {
            Err(libc::EINVAL)
        }
    }

    fn check_protocol(&self, protocol: &str) -> ZmqResult<()> {
        match protocol {
            "inproc" | "tcp" | "ipc" | "pgm" | "epgm" | "ws" | "wss" => Ok(()),
            _ => Err(libc::EPROTONOSUPPORT),
        }
    }
}

// Additional support structures
// struct Context {
//     options: HashMap<i32, i32>,
// }

struct Mailbox {
    thread_safe: bool,
    // ... mailbox implementation
}

struct Pipe {
    // ... pipe implementation
}

struct Endpoint {
    // ... endpoint implementation
}

struct Message {
    // ... message implementation
}

// Specific socket types
struct PairSocket {
    base: SocketBase,
}

struct PubSocket {
    base: SocketBase,
}

struct SubSocket {
    base: SocketBase,
}

// And so on for other socket types...

// Implementation of specific socket behaviors
impl SocketBehavior for PairSocket {
    // ... implement socket behavior for PAIR
}

impl SocketBehavior for PubSocket {
    // ... implement socket behavior for PUB
}

impl SocketBehavior for SubSocket {
    // ... implement socket behavior for SUB
}

// pub trait SocketOps {

// }
