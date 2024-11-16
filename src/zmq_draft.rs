#![allow(non_camel_case_types)]

use crate::{constants::ZMQ_EVENT_ALL, zmq_msg_t};

// Socket types
pub const ZMQ_SERVER: i32 = 12;
pub const ZMQ_CLIENT: i32 = 13;
pub const ZMQ_RADIO: i32 = 14;
pub const ZMQ_DISH: i32 = 15;
pub const ZMQ_GATHER: i32 = 16;
pub const ZMQ_SCATTER: i32 = 17;
pub const ZMQ_DGRAM: i32 = 18;
pub const ZMQ_PEER: i32 = 19;
pub const ZMQ_CHANNEL: i32 = 20;

// Socket options
pub const ZMQ_ZAP_ENFORCE_DOMAIN: i32 = 93;
pub const ZMQ_LOOPBACK_FASTPATH: i32 = 94;
pub const ZMQ_METADATA: i32 = 95;
pub const ZMQ_MULTICAST_LOOP: i32 = 96;
pub const ZMQ_ROUTER_NOTIFY: i32 = 97;
pub const ZMQ_XPUB_MANUAL_LAST_VALUE: i32 = 98;
pub const ZMQ_SOCKS_USERNAME: i32 = 99;
pub const ZMQ_SOCKS_PASSWORD: i32 = 100;
pub const ZMQ_IN_BATCH_SIZE: i32 = 101;
pub const ZMQ_OUT_BATCH_SIZE: i32 = 102;
pub const ZMQ_WSS_KEY_PEM: i32 = 103;
pub const ZMQ_WSS_CERT_PEM: i32 = 104;
pub const ZMQ_WSS_TRUST_PEM: i32 = 105;
pub const ZMQ_WSS_HOSTNAME: i32 = 106;
pub const ZMQ_WSS_TRUST_SYSTEM: i32 = 107;
pub const ZMQ_ONLY_FIRST_SUBSCRIBE: i32 = 108;
pub const ZMQ_RECONNECT_STOP: i32 = 109;
pub const ZMQ_HELLO_MSG: i32 = 110;
pub const ZMQ_DISCONNECT_MSG: i32 = 111;
pub const ZMQ_PRIORITY: i32 = 112;
pub const ZMQ_BUSY_POLL: i32 = 113;
pub const ZMQ_HICCUP_MSG: i32 = 114;
pub const ZMQ_XSUB_VERBOSE_UNSUBSCRIBE: i32 = 115;
pub const ZMQ_TOPICS_COUNT: i32 = 116;
pub const ZMQ_NORM_MODE: i32 = 117;
pub const ZMQ_NORM_UNICAST_NACK: i32 = 118;
pub const ZMQ_NORM_BUFFER_SIZE: i32 = 119;
pub const ZMQ_NORM_SEGMENT_SIZE: i32 = 120;
pub const ZMQ_NORM_BLOCK_SIZE: i32 = 121;
pub const ZMQ_NORM_NUM_PARITY: i32 = 122;
pub const ZMQ_NORM_NUM_AUTOPARITY: i32 = 123;
pub const ZMQ_NORM_PUSH: i32 = 124;

// NORM mode options
pub const ZMQ_NORM_FIXED: i32 = 0;
pub const ZMQ_NORM_CC: i32 = 1;
pub const ZMQ_NORM_CCL: i32 = 2;
pub const ZMQ_NORM_CCE: i32 = 3;
pub const ZMQ_NORM_CCE_ECNONLY: i32 = 4;

// Reconnect stop options
pub const ZMQ_RECONNECT_STOP_CONN_REFUSED: i32 = 0x1;
pub const ZMQ_RECONNECT_STOP_HANDSHAKE_FAILED: i32 = 0x2;
pub const ZMQ_RECONNECT_STOP_AFTER_DISCONNECT: i32 = 0x4;

// Context options
pub const ZMQ_ZERO_COPY_RECV: i32 = 10;

// Message property names
pub const ZMQ_MSG_PROPERTY_ROUTING_ID: &str = "Routing-Id";
pub const ZMQ_MSG_PROPERTY_SOCKET_TYPE: &str = "Socket-Type";
pub const ZMQ_MSG_PROPERTY_USER_ID: &str = "User-Id";
pub const ZMQ_MSG_PROPERTY_PEER_ADDRESS: &str = "Peer-Address";

// Router notify options
pub const ZMQ_NOTIFY_CONNECT: i32 = 1;
pub const ZMQ_NOTIFY_DISCONNECT: i32 = 2;

// Event monitoring constants
pub const ZMQ_EVENT_PIPES_STATS: u64 = 0x10000;
pub const ZMQ_CURRENT_EVENT_VERSION: i32 = 1;
pub const ZMQ_CURRENT_EVENT_VERSION_DRAFT: i32 = 2;

pub const ZMQ_EVENT_ALL_V1: u64 = ZMQ_EVENT_ALL;

pub const ZMQ_EVENT_ALL_V2: u64 = ZMQ_EVENT_ALL_V1 | ZMQ_EVENT_PIPES_STATS;

#[cfg(windows)]
pub type zmq_fd_t = libc::SOCKET;
#[cfg(not(windows))]
pub type zmq_fd_t = libc::c_int;

#[repr(C)]
pub struct zmq_poller_event_t {
    pub socket: *mut libc::c_void,
    pub fd: zmq_fd_t,
    pub user_data: *mut libc::c_void,
    pub events: libc::c_short,
}

#[link(name = "zmq")]
extern "C" {
    pub fn zmq_ctx_set_ext(
        context: *mut libc::c_void,
        option: libc::c_int,
        optval: *const libc::c_void,
        optvallen: libc::size_t,
    ) -> libc::c_int;

    pub fn zmq_ctx_get_ext(
        context: *mut libc::c_void,
        option: libc::c_int,
        optval: *mut libc::c_void,
        optvallen: *mut libc::size_t,
    ) -> libc::c_int;

    pub fn zmq_join(socket: *mut libc::c_void, group: *const libc::c_char) -> libc::c_int;
    pub fn zmq_leave(socket: *mut libc::c_void, group: *const libc::c_char) -> libc::c_int;

    pub fn zmq_msg_set_routing_id(msg: *mut zmq_msg_t, routing_id: u32) -> libc::c_int;
    pub fn zmq_msg_routing_id(msg: *mut zmq_msg_t) -> u32;
    pub fn zmq_msg_set_group(msg: *mut zmq_msg_t, group: *const libc::c_char) -> libc::c_int;
    pub fn zmq_msg_group(msg: *mut zmq_msg_t) -> *const libc::c_char;
    pub fn zmq_msg_init_buffer(
        msg: *mut zmq_msg_t,
        buf: *const libc::c_void,
        size: libc::size_t,
    ) -> libc::c_int;

    pub fn zmq_poller_new() -> *mut libc::c_void;
    pub fn zmq_poller_destroy(poller_p: *mut *mut libc::c_void) -> libc::c_int;
    pub fn zmq_poller_size(poller: *mut libc::c_void) -> libc::c_int;
    pub fn zmq_poller_add(
        poller: *mut libc::c_void,
        socket: *mut libc::c_void,
        user_data: *mut libc::c_void,
        events: libc::c_short,
    ) -> libc::c_int;
    pub fn zmq_poller_modify(
        poller: *mut libc::c_void,
        socket: *mut libc::c_void,
        events: libc::c_short,
    ) -> libc::c_int;
    pub fn zmq_poller_remove(poller: *mut libc::c_void, socket: *mut libc::c_void) -> libc::c_int;
    pub fn zmq_poller_wait(
        poller: *mut libc::c_void,
        event: *mut zmq_poller_event_t,
        timeout: libc::c_long,
    ) -> libc::c_int;
    pub fn zmq_poller_wait_all(
        poller: *mut libc::c_void,
        events: *mut zmq_poller_event_t,
        n_events: libc::c_int,
        timeout: libc::c_long,
    ) -> libc::c_int;
    pub fn zmq_poller_fd(poller: *mut libc::c_void) -> zmq_fd_t;
}

// Note: zmq_msg_t type needs to be defined elsewhere in your codebase
// as it's not part of the draft API
