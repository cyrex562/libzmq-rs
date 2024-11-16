use crate::types::ZmqSaFamily;

pub struct zmq_sockaddr_storage {
    pub ss_family: ZmqSaFamily,
    #[cfg(target_pointer_width = "32")]
    __ss_pad2: [u8; 128 - 2 - 4],
    #[cfg(target_pointer_width = "64")]
    __ss_pad2: [u8; 128 - 2 - 8],
    __ss_align: libc::size_t,
}

pub type zmq_socklen_t = u32;
