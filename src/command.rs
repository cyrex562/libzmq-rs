use crate::object::{Object, Own};
use crate::pipe::Pipe;
use crate::session_base::{Engine, SocketBase};
use std::ffi::c_int;
// Forward declarations for external types
// pub trait Object {}
// pub trait Own: Object {}
// pub trait Engine {}
// pub trait Pipe {}
// pub trait SocketBase {}

#[repr(C)]
#[derive(Debug)]
pub enum CommandType {
    Stop,
    Plug,
    Own,
    Attach,
    Bind,
    ActivateRead,
    ActivateWrite,
    Hiccup,
    PipeTerm,
    PipeTermAck,
    PipeHwm,
    TermReq,
    Term,
    TermAck,
    TermEndpoint,
    Reap,
    Reaped,
    InprocConnected,
    ConnFailed,
    PipePeerStats,
    PipeStatsPublish,
    Done,
}

#[derive(Debug)]
pub struct EndpointUriPair {
    // Implementation details to be added based on the original C++ endpoint_uri_pair_t
}

#[repr(C)]
#[derive(Debug)]
pub enum CommandArgs {
    Stop,
    Plug,
    Own {
        object: Box<dyn Own>,
    },
    Attach {
        engine: Option<Box<dyn Engine>>,
    },
    Bind {
        pipe: Box<dyn Pipe>,
    },
    ActivateRead,
    ActivateWrite {
        msgs_read: u64,
    },
    Hiccup {
        pipe: Box<dyn Pipe>,
    },
    PipeTerm,
    PipeTermAck,
    PipeHwm {
        inhwm: c_int,
        outhwm: c_int,
    },
    TermReq {
        object: Box<dyn Own>,
    },
    Term {
        linger: c_int,
    },
    TermAck,
    TermEndpoint {
        endpoint: String,
    },
    Reap {
        socket: Box<dyn SocketBase>,
    },
    Reaped,
    PipePeerStats {
        queue_count: u64,
        socket_base: Box<dyn Own>,
        endpoint_pair: Box<EndpointUriPair>,
    },
    PipeStatsPublish {
        outbound_queue_count: u64,
        inbound_queue_count: u64,
        endpoint_pair: Box<EndpointUriPair>,
    },
    Done,
}

#[repr(C)]
#[cfg_attr(target_os = "linux", repr(align(64)))] // ZMQ_CACHELINE_SIZE is typically 64
pub struct Command {
    pub destination: Box<dyn Object>,
    pub typ: CommandType,
    pub args: CommandArgs,
}
