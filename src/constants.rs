use libc::c_int;
use std::os::raw;

pub const ZMQ_PROBE_ROUTER: i32 = 51;

#[cfg(unix)]
pub const ZMQ_EINPROGRESS: c_int = libc::EINPROGRESS;
#[cfg(windows)]
pub const ZMQ_EINPROGRESS: c_int = winapi::um::winsock2::WSAEINPROGRESS;

#[cfg(unix)]
pub const ZMQ_EAGAIN: c_int = libc::EAGAIN;
#[cfg(windows)]
pub const ZMQ_EAGAIN: c_int = winapi::um::winsock2::WSAEWOULDBLOCK;

#[cfg(unix)]
pub const ZMQ_ENOPROTOOPT: c_int = libc::ENOPROTOOPT;
#[cfg(windows)]
pub const ZMQ_ENOPROTOOPT: c_int = winapi::um::winsock2::WSAENOPROTOOPT;

#[cfg(unix)]
pub const ZMQ_EINTR: c_int = libc::EINTR;
#[cfg(windows)]
pub const ZMQ_EINTR: c_int = winapi::um::winsock2::WSAEINTR;

#[cfg(unix)]
pub const ZMQ_EFSM: c_int = libc::EFSM;
#[cfg(windows)]
pub const ZMQ_EFSM: c_int = 10052;

#[cfg(unix)]
pub const ZMQ_EPROTO: c_int = libc::EPROTO;
#[cfg(windows)]
pub const ZMQ_EPROTO: c_int = 10053;

#[cfg(unix)]
pub const ZMQ_EFAULT: c_int = libc::EFAULT;
#[cfg(windows)]
pub const ZMQ_EFAULT: c_int = winapi::um::winsock2::WSAEFAULT;

// #define ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND 0x10000001
pub const ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND: i32 = 0x10000001;
// #define ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_WELCOME 0x10000017
pub const ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_WELCOME: i32 = 0x10000017;
// #define ZMQ_EVENT_HANDSHAKE_FAILED_AUTH 0x4000
pub const ZMQ_EVENT_HANDSHAKE_FAILED_AUTH: i32 = 0x4000;
// #define ZMQ_PROTOCOL_ERROR_ZMTP_UNSPECIFIED 0x10000000
pub const ZMQ_PROTOCOL_ERROR_ZMTP_UNSPECIFIED: i32 = 0x10000000;
// #define ZMQ_PROTOCOL_ERROR_ZMTP_INVALID_SEQUENCE 0x10000002
pub const ZMQ_PROTOCOL_ERROR_ZMTP_INVALID_SEQUENCE: i32 = 0x10000002;
// #define ZMQ_PROTOCOL_ERROR_ZMTP_KEY_EXCHANGE 0x10000003
pub const ZMQ_PROTOCOL_ERROR_ZMTP_KEY_EXCHANGE: i32 = 0x10000003;
// #define ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_UNSPECIFIED 0x10000011
pub const ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_UNSPECIFIED: i32 = 0x10000011;
// #define ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_MESSAGE 0x10000012
pub const ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_MESSAGE: i32 = 0x10000012;
// #define ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_HELLO 0x10000013
pub const ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_HELLO: i32 = 0x10000013;
// #define ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_INITIATE 0x10000014
pub const ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_INITIATE: i32 = 0x10000014;
// #define ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_ERROR 0x10000015
pub const ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_ERROR: i32 = 0x10000015;
// #define ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_READY 0x10000016
pub const ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_READY: i32 = 0x10000016;
// #define ZMQ_PROTOCOL_ERROR_ZMTP_INVALID_METADATA 0x10000018
pub const ZMQ_PROTOCOL_ERROR_ZMTP_INVALID_METADATA: i32 = 0x10000018;
// #define ZMQ_PROTOCOL_ERROR_ZMTP_CRYPTOGRAPHIC 0x11000001
pub const ZMQ_PROTOCOL_ERROR_ZMTP_CRYPTOGRAPHIC: i32 = 0x11000001;
// #define ZMQ_PROTOCOL_ERROR_ZMTP_MECHANISM_MISMATCH 0x11000002
pub const ZMQ_PROTOCOL_ERROR_ZMTP_MECHANISM_MISMATCH: i32 = 0x11000002;
// #define ZMQ_PROTOCOL_ERROR_ZAP_UNSPECIFIED 0x20000000
pub const ZMQ_PROTOCOL_ERROR_ZAP_UNSPECIFIED: i32 = 0x20000000;
// #define ZMQ_PROTOCOL_ERROR_ZAP_MALFORMED_REPLY 0x20000001
pub const ZMQ_PROTOCOL_ERROR_ZAP_MALFORMED_REPLY: i32 = 0x20000001;
// #define ZMQ_PROTOCOL_ERROR_ZAP_BAD_REQUEST_ID 0x20000002
pub const ZMQ_PROTOCOL_ERROR_ZAP_BAD_REQUEST_ID: i32 = 0x20000002;
// #define ZMQ_PROTOCOL_ERROR_ZAP_BAD_VERSION 0x20000003
pub const ZMQ_PROTOCOL_ERROR_ZAP_BAD_VERSION: i32 = 0x20000003;
// #define ZMQ_PROTOCOL_ERROR_ZAP_INVALID_STATUS_CODE 0x20000004
pub const ZMQ_PROTOCOL_ERROR_ZAP_INVALID_STATUS_CODE: i32 = 0x20000004;
// #define ZMQ_PROTOCOL_ERROR_ZAP_INVALID_METADATA 0x20000005
pub const ZMQ_PROTOCOL_ERROR_ZAP_INVALID_METADATA: i32 = 0x20000005;
// #define ZMQ_PROTOCOL_ERROR_WS_UNSPECIFIED 0x30000000
pub const ZMQ_PROTOCOL_ERROR_WS_UNSPECIFIED: i32 = 0x30000000;

// Constants from zmq.h would go here
pub const ZMQ_VERSION_MAJOR: i32 = 4;
pub const ZMQ_VERSION_MINOR: i32 = 3;
pub const ZMQ_VERSION_PATCH: i32 = 4;

#[cfg(feature = "draft")]
pub const ZMQ_BUILD_DRAFT_API: bool = true;

#[cfg(unix)]
pub const ZMQ_FD_ZERO: c_int = libc::FD_ZERO;
#[cfg(windows)]
pub const ZMQ_FD_ZERO: c_int = 10054;

#[cfg(unix)]
pub const ZMQ_FD_CLR: c_int = libc::FD_CLR;
#[cfg(windows)]
pub const ZMQ_FD_CLR: c_int = 10055;

// AF_UNSPEC
#[cfg(unix)]
pub const ZMQ_AF_UNSPEC: c_int = libc::AF_UNSPEC;
#[cfg(windows)]
pub const ZMQ_AF_UNSPEC: c_int = 0;

// FD_SET
#[cfg(unix)]
pub const ZMQ_FD_SET: c_int = libc::FD_SET;
#[cfg(windows)]
pub const ZMQ_FD_SET: c_int = 10053;

// SOL_SOCKET
#[cfg(unix)]
pub const ZMQ_SOL_SOCKET: c_int = libc::SOL_SOCKET;
#[cfg(windows)]
pub const ZMQ_SOL_SOCKET: c_int = 0xffff;

// SO_TYPE
#[cfg(unix)]
pub const ZMQ_SO_TYPE: c_int = libc::SO_TYPE;
#[cfg(windows)]
pub const ZMQ_SO_TYPE: c_int = 4104;

// SOCK_DGRAM
#[cfg(unix)]
pub const ZMQ_SOCK_DGRAM: c_int = libc::SOCK_DGRAM;
#[cfg(windows)]
pub const ZMQ_SOCK_DGRAM: c_int = 2;

// #define ZMQ_EVENT_CONNECTED 0x0001
pub const ZMQ_EVENT_CONNECTED: i32 = 0x0001;
// #define ZMQ_EVENT_CONNECT_DELAYED 0x0002
pub const ZMQ_EVENT_CONNECT_DELAYED: i32 = 0x0002;
// #define ZMQ_EVENT_CONNECT_RETRIED 0x0004
pub const ZMQ_EVENT_CONNECT_RETRIED: i32 = 0x0004;
// #define ZMQ_EVENT_LISTENING 0x0008
pub const ZMQ_EVENT_LISTENING: i32 = 0x0008;
// #define ZMQ_EVENT_BIND_FAILED 0x0010
pub const ZMQ_EVENT_BIND_FAILED: i32 = 0x0010;
// #define ZMQ_EVENT_ACCEPTED 0x0020
pub const ZMQ_EVENT_ACCEPTED: i32 = 0x0020;
// #define ZMQ_EVENT_ACCEPT_FAILED 0x0040
pub const ZMQ_EVENT_ACCEPT_FAILED: i32 = 0x0040;
// #define ZMQ_EVENT_CLOSED 0x0080
pub const ZMQ_EVENT_CLOSED: i32 = 0x0080;
// #define ZMQ_EVENT_CLOSE_FAILED 0x0100
pub const ZMQ_EVENT_CLOSE_FAILED: i32 = 0x0100;
// #define ZMQ_EVENT_DISCONNECTED 0x0200
pub const ZMQ_EVENT_DISCONNECTED: i32 = 0x0200;
// #define ZMQ_EVENT_MONITOR_STOPPED 0x0400
pub const ZMQ_EVENT_MONITOR_STOPPED: i32 = 0x0400;
// #define ZMQ_EVENT_ALL 0xFFFF
pub const ZMQ_EVENT_ALL: i32 = 0xFFFF;

/*  Socket options.                                                           */
// #define ZMQ_AFFINITY 4
pub const ZMQ_AFFINITY: i32 = 4;
// #define ZMQ_ROUTING_ID 5
pub const ZMQ_ROUTING_ID: i32 = 5;
// #define ZMQ_SUBSCRIBE 6
pub const ZMQ_SUBSCRIBE: i32 = 6;
// #define ZMQ_UNSUBSCRIBE 7
pub const ZMQ_UNSUBSCRIBE: i32 = 7;
// #define ZMQ_RATE 8
pub const ZMQ_RATE: i32 = 8;
// #define ZMQ_RECOVERY_IVL 9
pub const ZMQ_RECOVERY_IVL: i32 = 9;
// #define ZMQ_SNDBUF 11
pub const ZMQ_SNDBUF: i32 = 11;
// #define ZMQ_RCVBUF 12
pub const ZMQ_RCVBUF: i32 = 12;
// #define ZMQ_RCVMORE 13
pub const ZMQ_RECV_MORE: i32 = 13;
// #define ZMQ_FD 14
pub const ZMQ_FD: i32 = 14;
// #define ZMQ_EVENTS 15
pub const ZMQ_EVENTS: i32 = 15;
// #define ZMQ_TYPE 16
pub const ZMQ_TYPE: i32 = 16;
// #define ZMQ_LINGER 17
pub const ZMQ_LINGER: i32 = 17;
// #define ZMQ_RECONNECT_IVL 18
pub const ZMQ_RECONNECT_IVL: i32 = 18;
// #define ZMQ_BACKLOG 19
pub const ZMQ_BACKLOG: i32 = 19;
// #define ZMQ_RECONNECT_IVL_MAX 21
pub const ZMQ_RECONNECT_IVL_MAX: i32 = 21;
// #define ZMQ_MAXMSGSIZE 22
pub const ZMQ_MAXMSGSIZE: i32 = 22;
// #define ZMQ_SNDHWM 23
pub const ZMQ_SNDHWM: i32 = 23;
// #define ZMQ_RCVHWM 24
pub const ZMQ_RCVHWM: i32 = 24;
// #define ZMQ_MULTICAST_HOPS 25
pub const ZMQ_MULTICAST_HOPS: i32 = 25;
// #define ZMQ_RCVTIMEO 27
pub const ZMQ_RCVTIMEO: i32 = 27;
// #define ZMQ_SNDTIMEO 28
pub const ZMQ_SNDTIMEO: i32 = 28;
// #define ZMQ_LAST_ENDPOINT 32
pub const ZMQ_LAST_ENDPOINT: i32 = 32;
// #define ZMQ_ROUTER_MANDATORY 33
pub const ZMQ_ROUTER_MANDATORY: i32 = 33;
// #define ZMQ_TCP_KEEPALIVE 34
pub const ZMQ_TCP_KEEPALIVE: i32 = 34;
// #define ZMQ_TCP_KEEPALIVE_CNT 35
pub const ZMQ_TCP_KEEPALIVE_CNT: i32 = 35;
// #define ZMQ_TCP_KEEPALIVE_IDLE 36
pub const ZMQ_TCP_KEEPALIVE_IDLE: i32 = 36;
// #define ZMQ_TCP_KEEPALIVE_INTVL 37
pub const ZMQ_TCP_KEEPALIVE_INTVL: i32 = 37;
// #define ZMQ_IMMEDIATE 39
pub const ZMQ_IMMEDIATE: i32 = 39;
// #define ZMQ_XPUB_VERBOSE 40
pub const ZMQ_XPUB_VERBOSE: i32 = 40;
// #define ZMQ_ROUTER_RAW 41
pub const ZMQ_ROUTER_RAW: i32 = 41;
// #define ZMQ_IPV6 42
pub const ZMQ_IPV6: i32 = 42;
// #define ZMQ_MECHANISM 43
pub const ZMQ_MECHANISM: i32 = 43;
// #define ZMQ_PLAIN_SERVER 44
pub const ZMQ_PLAIN_SERVER: i32 = 44;
// #define ZMQ_PLAIN_USERNAME 45
pub const ZMQ_PLAIN_USERNAME: i32 = 45;
// #define ZMQ_PLAIN_PASSWORD 46
pub const ZMQ_PLAIN_PASSWORD: i32 = 46;
// #define ZMQ_CURVE_SERVER 47
pub const ZMQ_CURVE_SERVER: i32 = 47;
// #define ZMQ_CURVE_PUBLICKEY 48
pub const ZMQ_CURVE_PUBLICKEY: i32 = 48;
// #define ZMQ_CURVE_SECRETKEY 49
pub const ZMQ_CURVE_SECRETKEY: i32 = 49;
// #define ZMQ_CURVE_SERVERKEY 50
pub const ZMQ_CURVE_SERVERKEY: i32 = 50;
// #define ZMQ_PROBE_ROUTER 51
// pub const ZMQ_PROBE_ROUTER: i32 = 51;
// #define ZMQ_REQ_CORRELATE 52
pub const ZMQ_REQ_CORRELATE: i32 = 52;
// #define ZMQ_REQ_RELAXED 53
pub const ZMQ_REQ_RELAXED: i32 = 53;
// #define ZMQ_CONFLATE 54
pub const ZMQ_CONFLATE: i32 = 54;
// #define ZMQ_ZAP_DOMAIN 55
pub const ZMQ_ZAP_DOMAIN: i32 = 55;
// #define ZMQ_ROUTER_HANDOVER 56
pub const ZMQ_ROUTER_HANDOVER: i32 = 56;
// #define ZMQ_TOS 57
pub const ZMQ_TOS: i32 = 57;
// #define ZMQ_CONNECT_ROUTING_ID 61
pub const ZMQ_CONNECT_ROUTING_ID: i32 = 61;
// #define ZMQ_GSSAPI_SERVER 62
pub const ZMQ_GSSAPI_SERVER: i32 = 62;
// #define ZMQ_GSSAPI_PRINCIPAL 63
pub const ZMQ_GSSAPI_PRINCIPAL: i32 = 63;
// #define ZMQ_GSSAPI_SERVICE_PRINCIPAL 64
pub const ZMQ_GSSAPI_SERVICE_PRINCIPAL: i32 = 64;
// #define ZMQ_GSSAPI_PLAINTEXT 65
pub const ZMQ_GSSAPI_PLAINTEXT: i32 = 65;
// #define ZMQ_HANDSHAKE_IVL 66
pub const ZMQ_HANDSHAKE_IVL: i32 = 66;
// #define ZMQ_SOCKS_PROXY 68
pub const ZMQ_SOCKS_PROXY: i32 = 68;
// #define ZMQ_XPUB_NODROP 69
pub const ZMQ_XPUB_NODROP: i32 = 69;
// #define ZMQ_BLOCKY 70
pub const ZMQ_BLOCKY: i32 = 70;
// #define ZMQ_XPUB_MANUAL 71
pub const ZMQ_XPUB_MANUAL: i32 = 71;
// #define ZMQ_XPUB_WELCOME_MSG 72
pub const ZMQ_XPUB_WELCOME_MSG: i32 = 72;
// #define ZMQ_STREAM_NOTIFY 73
pub const ZMQ_STREAM_NOTIFY: i32 = 73;
// #define ZMQ_INVERT_MATCHING 74
pub const ZMQ_INVERT_MATCHING: i32 = 74;
// #define ZMQ_HEARTBEAT_IVL 75
pub const ZMQ_HEARTBEAT_IVL: i32 = 75;
// #define ZMQ_HEARTBEAT_TTL 76
pub const ZMQ_HEARTBEAT_TTL: i32 = 76;
// #define ZMQ_HEARTBEAT_TIMEOUT 77
pub const ZMQ_HEARTBEAT_TIMEOUT: i32 = 77;
// #define ZMQ_XPUB_VERBOSER 78
pub const ZMQ_XPUB_VERBOSER: i32 = 78;
// #define ZMQ_CONNECT_TIMEOUT 79
pub const ZMQ_CONNECT_TIMEOUT: i32 = 79;
// #define ZMQ_TCP_MAXRT 80
pub const ZMQ_TCP_MAXRT: i32 = 80;
// #define ZMQ_THREAD_SAFE 81
pub const ZMQ_THREAD_SAFE: i32 = 81;
// #define ZMQ_MULTICAST_MAXTPDU 84
pub const ZMQ_MULTICAST_MAXTPDU: i32 = 84;
// #define ZMQ_VMCI_BUFFER_SIZE 85
pub const ZMQ_VMCI_BUFFER_SIZE: i32 = 85;
// #define ZMQ_VMCI_BUFFER_MIN_SIZE 86
pub const ZMQ_VMCI_BUFFER_MIN_SIZE: i32 = 86;
// #define ZMQ_VMCI_BUFFER_MAX_SIZE 87
pub const ZMQ_VMCI_BUFFER_MAX_SIZE: i32 = 87;
// #define ZMQ_VMCI_CONNECT_TIMEOUT 88
pub const ZMQ_VMCI_CONNECT_TIMEOUT: i32 = 88;
// #define ZMQ_USE_FD 89
pub const ZMQ_USE_FD: i32 = 89;
// #define ZMQ_GSSAPI_PRINCIPAL_NAMETYPE 90
pub const ZMQ_GSSAPI_PRINCIPAL_NAMETYPE: i32 = 90;
// #define ZMQ_GSSAPI_SERVICE_PRINCIPAL_NAMETYPE 91
pub const ZMQ_GSSAPI_SERVICE_PRINCIPAL_NAMETYPE: i32 = 91;
// #define ZMQ_BINDTODEVICE 92
pub const ZMQ_BINDTODEVICE: i32 = 92;

#[cfg(unix)]
pub const ZMQ_ETERM: c_int = libc::ETERM;
#[cfg(windows)]
pub const ZMQ_ETERM: c_int = winapi::um::winsock2::WSAESHUTDOWN;

#[cfg(unix)]
pub const ZMQ_EPROTONOSUPPORT: c_int = libc::EPROTONOSUPPORT;
#[cfg(windows)]
pub const ZMQ_EPROTONOSUPPORT: c_int = winapi::um::winsock2::WSAEPROTONOSUPPORT;

// #[cfg(unix)]
// pub type ZmqSockAddrIn6 = libc::sockaddr_in6;
// #[cfg(windows)]
// pub type ZmqSockAddrIn6 = winapi::shared::ws2ipdef::SOCKADDR_IN6;
// 
// #[cfg(unix)]
// pub type ZmqSockAddrIn = libc::sockaddr_in;
// 
// #[cfg(windows)]
// pub type ZmqSockAddrIn = winapi::shared::ws2def::SOCKADDR_IN;


// Version info
// pub const ZMQ_VERSION_MAJOR: raw::c_int = 4;
// pub const ZMQ_VERSION_MINOR: raw::c_int = 3;
// pub const ZMQ_VERSION_PATCH: raw::c_int = 6;
// Socket types
pub const ZMQ_PAIR: raw::c_int = 0;
pub const ZMQ_PUB: raw::c_int = 1;
pub const ZMQ_SUB: raw::c_int = 2;
pub const ZMQ_REQ: raw::c_int = 3;
pub const ZMQ_REP: raw::c_int = 4;
pub const ZMQ_DEALER: raw::c_int = 5;
pub const ZMQ_ROUTER: raw::c_int = 6;
pub const ZMQ_PULL: raw::c_int = 7;
pub const ZMQ_PUSH: raw::c_int = 8;
pub const ZMQ_XPUB: raw::c_int = 9;
pub const ZMQ_XSUB: raw::c_int = 10;
pub const ZMQ_STREAM: raw::c_int = 11;
// Error codes
pub const EFAULT: i32 = 14;