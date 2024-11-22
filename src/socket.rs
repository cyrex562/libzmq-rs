use crate::types::ZmqSaFamily;

#[cfg(unix)]
pub type ZmqPlatformSocket = libc::c_int;
#[cfg(windows)]
pub type ZmqPlatformSocket = libc::SOCKET;
// pub type ZmqSockAddrIn = libc::sockaddr_in;
// pub type ZmqSockAddrIn6 = libc::sockaddr_in6;

#[derive(Debug, Copy, Clone)]
pub struct ZmqSockAddrIn {
    family: ZmqSaFamily,
    port: u16,
    addr: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct ZmqSockAddrIn6 {
    family: ZmqSaFamily,
    port: u16,
    flow_info: u32,
    addr: [u8;16],
    scope_id: u32
}

pub struct ZmqSockAddr {
    family: ZmqSaFamily,
    data: [u8;14]
}
