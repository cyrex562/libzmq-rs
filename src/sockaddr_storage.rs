use std::ffi::c_int;
use crate::types::ZmqSaFamily;

pub struct ZmqSockaddrStorage {
    pub ss_family: ZmqSaFamily,
    #[cfg(target_pointer_width = "32")]
    pub __ss_pad2: [u8; 128 - 2 - 4],
    #[cfg(target_pointer_width = "64")]
    pub(crate) __ss_pad2: [u8; 128 - 2 - 8],
    pub __ss_align: libc::size_t,
}

impl ZmqSockaddrStorage {
    pub fn new() -> Self {
        ZmqSockaddrStorage {
            ss_family: 0,
            #[cfg(target_pointer_width = "32")]
            __ss_pad2: [0; 122],
            #[cfg(target_pointer_width = "64")]
            __ss_pad2: [0; 118],
            __ss_align: 0,
        }
    }
}

pub type ZmqSocklen = u32;

pub fn zmq_socklen_to_c_int(len: ZmqSocklen) -> c_int {
    len as c_int
}