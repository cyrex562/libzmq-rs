
use std::ptr::null_mut;
use std::os::raw::c_void;
use crate::ctx::Ctx;
use crate::tcp_address::TcpAddress;
use crate::udp_address::UdpAddress;
// mod ctx;
// mod tcp_address;
// mod udp_address;
// mod ipc_address;
// mod tipc_address;
// mod ws_address;

#[cfg(feature = "vmci")]
mod vmci_address;

pub struct Address {
    protocol: String,
    address: String,
    parent: *mut Ctx,
    resolved: ResolvedAddress,
}

enum ResolvedAddress {
    Dummy(*mut c_void),
    Tcp(*mut TcpAddress),
    Udp(*mut UdpAddress),
    #[cfg(feature = "ws")]
    Ws(*mut ws_address::WsAddress),
    #[cfg(feature = "wss")]
    Wss(*mut ws_address::WsAddress),
    #[cfg(feature = "ipc")]
    Ipc(*mut ipc_address::IpcAddress),
    #[cfg(any(feature = "linux", feature = "vxworks"))]
    Tipc(*mut tipc_address::TipcAddress),
    #[cfg(feature = "vmci")]
    Vmci(*mut vmci_address::VmciAddress),
}

impl Address {
    pub fn new(protocol: &str, address: &str, parent: *mut Ctx) -> Address {
        Address {
            protocol: protocol.to_string(),
            address: address.to_string(),
            parent,
            resolved: ResolvedAddress::Dummy(null_mut()),
        }
    }

    pub fn to_string(&self) -> Result<String, ()> {
        match self.protocol.as_str() {
            "tcp" => {
                if let ResolvedAddress::Tcp(addr) = self.resolved {
                    unsafe { (*addr).to_string() }
                } else {
                    Err(())
                }
            }
            "udp" => {
                if let ResolvedAddress::Udp(addr) = self.resolved {
                    unsafe { (*addr).to_string() }
                } else {
                    Err(())
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

impl Drop for Address {
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

pub fn get_socket_address(fd: i32, socket_end: SocketEnd, ss: &mut libc::sockaddr_storage) -> usize {
    let mut sl = std::mem::size_of::<libc::sockaddr_storage>() as libc::socklen_t;

    let rc = match socket_end {
        SocketEnd::Local => unsafe { libc::getsockname(fd, ss as *mut _ as *mut _, &mut sl) },
        SocketEnd::Remote => unsafe { libc::getpeername(fd, ss as *mut _ as *mut _, &mut sl) },
    };

    if rc != 0 {
        0
    } else {
        sl as usize
    }
}

pub enum SocketEnd {
    Local,
    Remote,
}

pub fn get_socket_name<T: ToString>(fd: i32, socket_end: SocketEnd) -> String {
    let mut ss: libc::sockaddr_storage = unsafe { std::mem::zeroed() };
    let sl = get_socket_address(fd, socket_end, &mut ss);
    if sl == 0 {
        return String::new();
    }

    let addr = T::from_sockaddr(&ss, sl);
    addr.to_string()
}
