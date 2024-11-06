use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct UdpAddress {
    bind_address: SocketAddr,
    bind_interface: i32,
    target_address: SocketAddr,
    is_multicast: bool,
    address: String,
}

impl Default for UdpAddress {
    fn default() -> Self {
        Self {
            // Default to 0.0.0.0:0
            bind_address: SocketAddr::from(([0, 0, 0, 0], 0)),
            bind_interface: -1,
            target_address: SocketAddr::from(([0, 0, 0, 0], 0)),
            is_multicast: false,
            address: String::new(),
        }
    }
}

impl UdpAddress {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn resolve(&mut self, name: &str, bind: bool, ipv6: bool) -> Result<(), std::io::Error> {
        self.address = name.to_string();

        // Handle interface specification (after semicolon)
        let (bind_part, addr_part) = if let Some(idx) = name.rfind(';') {
            let (src, dst) = name.split_at(idx);
            (Some(src), &dst[1..])
        } else {
            (None, name)
        };

        // Resolve the target address
        let target_addr = SocketAddr::from_str(addr_part)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

        let is_multicast = match target_addr.ip() {
            IpAddr::V4(addr) => addr.is_multicast(),
            IpAddr::V6(addr) => addr.is_multicast(),
        };

        if ipv6 && target_addr.is_ipv4() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "IPv6 requested but address is IPv4",
            ));
        }

        // Handle binding address
        if let Some(bind_addr) = bind_part {
            if bind_addr == "*" {
                self.bind_interface = 0;
                self.bind_address = if ipv6 {
                    SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 0], target_addr.port()))
                } else {
                    SocketAddr::from(([0, 0, 0, 0], target_addr.port()))
                };
            } else {
                #[cfg(unix)]
                {
                    use std::ffi::CString;
                    use std::os::unix::ffi::OsStrExt;
                    use std::path::Path;
                    
                    let if_name = Path::new(bind_addr);
                    if let Ok(if_name_cstr) = CString::new(if_name.as_os_str().as_bytes()) {
                        unsafe {
                            self.bind_interface = libc::if_nametoindex(if_name_cstr.as_ptr()) as i32;
                        }
                    }
                }
                
                if self.bind_interface == 0 {
                    self.bind_interface = -1;
                }
            }
        } else {
            if is_multicast || !bind {
                self.bind_address = if ipv6 {
                    SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 0], target_addr.port()))
                } else {
                    SocketAddr::from(([0, 0, 0, 0], target_addr.port()))
                };
                self.bind_interface = 0;
            } else {
                self.bind_address = target_addr;
            }
        }

        self.target_address = target_addr;
        self.is_multicast = is_multicast;

        if ipv6 && is_multicast && self.bind_interface < 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No interface specified for IPv6 multicast",
            ));
        }

        Ok(())
    }

    pub fn to_string(&self) -> String {
        self.address.clone()
    }

    pub fn family(&self) -> i32 {
        if self.bind_address.is_ipv4() { 2 } else { 10 }  // AF_INET or AF_INET6
    }

    pub fn is_mcast(&self) -> bool {
        self.is_multicast
    }

    pub fn bind_addr(&self) -> &SocketAddr {
        &self.bind_address
    }

    pub fn bind_if(&self) -> i32 {
        self.bind_interface
    }

    pub fn target_addr(&self) -> &SocketAddr {
        &self.target_address
    }
}
