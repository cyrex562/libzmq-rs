use crate::context::Context;
use crate::err::ZmqError;
#[cfg(all(feature = "ipc", unix))]
use crate::ipc_address;
use crate::sockaddr_storage::{zmq_socklen_to_c_int, ZmqSockaddrStorage, ZmqSocklen};
use crate::socket::{ZmqPlatformSocket, ZmqSockAddrIn, ZmqSockAddrIn6};
use crate::tcp_address::TcpAddress;
#[cfg(unix)]
use crate::tipc_address;
use crate::types::ZmqSaFamily;
use crate::udp_address::UdpAddress;
#[cfg(feature = "vmci")]
use crate::vmci_address;
#[cfg(feature = "ws")]
use crate::ws_address;
use std::os::raw::c_void;
use std::ptr::null_mut;

pub struct Address<'a> {
    protocol: String,
    address: String,
    parent: &'a mut Context,
    resolved: ResolvedAddress<'a>,
}

enum ResolvedAddress<'a> {
    Dummy(*mut c_void),
    Tcp(&'a mut TcpAddress),
    Udp(&'a mut UdpAddress),
    #[cfg(feature = "ws")]
    Ws(*mut ws_address::WsAddress),
    #[cfg(feature = "wss")]
    Wss(*mut ws_address::WsAddress),
    #[cfg(all(feature = "ipc", unix))]
    Ipc(*mut ipc_address::IpcAddress),
    #[cfg(all(feature = "tipc", target_os = "linux"))]
    Tipc(*mut tipc_address::TipcAddress),
    #[cfg(feature = "vmci")]
    Vmci(*mut vmci_address::VmciAddress),
}

impl<'a> Address<'a> {
    pub fn new(protocol: &str, address: &str, parent: &mut Context) -> Address<'a> {
        Address {
            protocol: protocol.to_string(),
            address: address.to_string(),
            parent,
            resolved: ResolvedAddress::Dummy(null_mut()),
        }
    }

    pub fn to_string(&self) -> Result<String, ZmqError> {
        match self.protocol.as_str() {
            "tcp" => {
                if let ResolvedAddress::Tcp(addr) = self.resolved {
                    addr.to_string()
                } else {
                    Err(ZmqError::ParsingError("Invalid TCP address".to_string()))
                }
            }
            "udp" => {
                if let ResolvedAddress::Udp(addr) = self.resolved {
                    addr.to_string()
                } else {
                    Err(ZmqError::ParsingError("Invalid UDP address".to_string()))
                }
            }
            #[cfg(feature = "ws")]
            "ws" => {
                if let ResolvedAddress::Ws(addr) = self.resolved {
                    unsafe { (*addr).to_string() }
                } else {
                    Err(())
                }
            }
            #[cfg(feature = "wss")]
            "wss" => {
                if let ResolvedAddress::Wss(addr) = self.resolved {
                    unsafe { (*addr).to_string() }
                } else {
                    Err(())
                }
            }
            #[cfg(feature = "ipc")]
            "ipc" => {
                if let ResolvedAddress::Ipc(addr) = self.resolved {
                    unsafe { (*addr).to_string() }
                } else {
                    Err(())
                }
            }
            #[cfg(any(feature = "linux", feature = "vxworks"))]
            "tipc" => {
                if let ResolvedAddress::Tipc(addr) = self.resolved {
                    unsafe { (*addr).to_string() }
                } else {
                    Err(())
                }
            }
            #[cfg(feature = "vmci")]
            "vmci" => {
                if let ResolvedAddress::Vmci(addr) = self.resolved {
                    unsafe { (*addr).to_string() }
                } else {
                    Err(())
                }
            }
            _ => {
                if !self.protocol.is_empty() && !self.address.is_empty() {
                    Ok(format!("{}://{}", self.protocol, self.address))
                } else {
                    Err(())
                }
            }
        }
    }
}

impl Drop for Address<'_> {
    fn drop(&mut self) {
        match self.protocol.as_str() {
            "tcp" => {
                if let ResolvedAddress::Tcp(addr) = self.resolved {
                    unsafe { Box::from_raw(addr) };
                }
            }
            "udp" => {
                if let ResolvedAddress::Udp(addr) = self.resolved {
                    unsafe { Box::from_raw(addr) };
                }
            }
            #[cfg(feature = "ws")]
            "ws" => {
                if let ResolvedAddress::Ws(addr) = self.resolved {
                    unsafe { Box::from_raw(addr) };
                }
            }
            #[cfg(feature = "wss")]
            "wss" => {
                if let ResolvedAddress::Wss(addr) = self.resolved {
                    unsafe { Box::from_raw(addr) };
                }
            }
            #[cfg(feature = "ipc")]
            "ipc" => {
                if let ResolvedAddress::Ipc(addr) = self.resolved {
                    unsafe { Box::from_raw(addr) };
                }
            }
            #[cfg(any(feature = "linux", feature = "vxworks"))]
            "tipc" => {
                if let ResolvedAddress::Tipc(addr) = self.resolved {
                    unsafe { Box::from_raw(addr) };
                }
            }
            #[cfg(feature = "vmci")]
            "vmci" => {
                if let ResolvedAddress::Vmci(addr) = self.resolved {
                    unsafe { Box::from_raw(addr) };
                }
            }
            _ => {}
        }
    }
}

pub fn get_socket_address(
    fd: ZmqPlatformSocket,
    socket_end: SocketEnd,
    ss: &mut ZmqSockaddrStorage,
) -> usize {
    let mut sock_len = size_of::<ZmqSockaddrStorage>() as ZmqSocklen;
    let mut sock_len_c_int = zmq_socklen_to_c_int(sock_len);

    let rc = match socket_end {
        SocketEnd::Local => unsafe {
            libc::getsockname(fd, ss as *mut _ as *mut _, &mut sock_len_c_int)
        },
        SocketEnd::Remote => unsafe {
            libc::getpeername(fd, ss as *mut _ as *mut _, &mut sock_len_c_int)
        },
    };

    if rc != 0 {
        0
    } else {
        sock_len as usize
    }
}

pub enum SocketEnd {
    Local,
    Remote,
}

pub fn get_socket_name(fd: ZmqPlatformSocket, socket_end: SocketEnd) -> Result<String, ZmqError> {
    // TODO: i believe this part is getting the tcp address in the form tcp://<addr>:port?
    let mut ss: ZmqSockaddrStorage = ZmqSockaddrStorage::new();
    let socklen = get_socket_address(fd, socket_end, &mut ss);
    if socklen == 0 {
        return Err(ZmqError::ParsingError(
            "Failed to get socket address".to_string(),
        ));
    }

    match ss.ss_family {
        ZmqSaFamily::AF_INET => {
            let sockaddr = ss.__ss_pad2.as_mut_ptr() as *mut ZmqSockAddrIn;
            let tcp_addr = TcpAddress::from_sockaddr(sockaddr);

            let sa = unsafe { *(ss.__ss_pad2.as_ptr() as *const ZmqSockAddrIn) };
            let addr = TcpAddress::from_sockaddr(&sa);
            addr.to_string()
        }
        ZmqSaFamily::AF_INET6 => {
            let sai = unsafe { *(ss.__ss_pad2.as_ptr() as *const ZmqSockAddrIn6) };
            let tcp_addr = TcpAddress::from_sockaddr_in6(&sai);
            tcp_addr.to_string()
        }
        _ => Err(ZmqError::ParsingError(
            "Unsupported address family".to_string(),
        )),
    }
}
