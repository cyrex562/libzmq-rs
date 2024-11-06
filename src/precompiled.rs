#![allow(unused_imports)]

// Standard library imports
use std::cmp;
use std::collections::{HashMap, HashSet, VecDeque};
use std::convert::TryFrom;
use std::error::Error;
use std::ffi::{c_void, CStr, CString};
use std::fmt;
use std::io;
use std::mem;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::os::raw::{c_char, c_int, c_long, c_uint};
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

// Platform-specific imports
#[cfg(windows)]
use winapi::{
    shared::{mswsock, winerror, ws2def, ws2ipdef},
    um::{iphlpapi, winsock2, ws2tcpip},
};

#[cfg(unix)]
use libc::{
    poll, pollfd, socket, AF_INET, AF_INET6, IPPROTO_TCP, SOCK_STREAM, SOL_SOCKET, SO_REUSEADDR,
};

// OpenBSD specific type alias
#[cfg(target_os = "openbsd")]
type Ucred = libc::sockpeercred;

// GSSAPI support
#[cfg(feature = "gssapi")]
use gssapi::{
    error::Error as GssApiError,
    credential::Credential,
    name::Name,
    context::{ClientCtx, ServerCtx},
};

// Constants from zmq.h would go here
pub const ZMQ_VERSION_MAJOR: i32 = 4;
pub const ZMQ_VERSION_MINOR: i32 = 3;
pub const ZMQ_VERSION_PATCH: i32 = 4;

// Draft API feature flag
#[cfg(feature = "draft")]
pub const ZMQ_BUILD_DRAFT_API: bool = true;

// Module declarations
pub mod error;
pub mod message;
pub mod socket;
pub mod context;
pub mod mechanism;
pub mod session;

// Re-exports
pub use error::Error as ZmqError;
pub use message::Message;
pub use socket::Socket;
pub use context::Context;
