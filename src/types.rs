#[cfg(unix)]
use std::os::unix::io::{AsRawFd, RawFd};

#[cfg(unix)]
pub type ZmqRawFd = RawFd;
#[cfg(windows)]
pub type ZmqRawFd = std::os::windows::io::RawSocket;

#[cfg(unix)]
pub type ZmqSocklen = libc::socklen_t;
#[cfg(windows)]
pub type ZmqSocklen = u32;

#[cfg(unix)]
pub type ZmqSaFamily = libc::sa_family_t;
#[cfg(windows)]
pub type ZmqSaFamily = winapi::shared::ws2def::ADDRESS_FAMILY;
