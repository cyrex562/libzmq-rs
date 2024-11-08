use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::str::FromStr;
use std::ffi::CStr;
use std::os::raw::c_char;

#[cfg(unix)]
use std::net::Ipv6Addr;
#[cfg(unix)]
use libc::{self, sockaddr, sockaddr_in, sockaddr_in6, AF_INET, AF_INET6};
use windows_sys::Win32::Networking::WinSock::{AF_INET, AF_INET6};

#[derive(Clone, Debug)]
pub struct IpAddrT {
    inner: SocketAddr,
}

impl IpAddrT {
    pub fn family(&self) -> i32 {
        match self.inner {
            SocketAddr::V4(_) => AF_INET as i32,
            SocketAddr::V6(_) => AF_INET6 as i32,
        }
    }

    pub fn is_multicast(&self) -> bool {
        match self.inner.ip() {
            IpAddr::V4(addr) => addr.is_multicast(),
            IpAddr::V6(addr) => addr.is_multicast(),
        }
    }

    pub fn port(&self) -> u16 {
        self.inner.port()
    }

    pub fn set_port(&mut self, port: u16) {
        self.inner.set_port(port);
    }

    pub fn any(family: i32) -> Self {
        let addr = match family {
            x if x == AF_INET as i32 => SocketAddr::from(([0, 0, 0, 0], 0)),
            x if x == AF_INET6 as i32 => SocketAddr::from(([0; 16], 0)),
            _ => panic!("Unsupported address family"),
        };
        IpAddrT { inner: addr }
    }
}

#[derive(Debug, Clone)]
pub struct IpResolverOptions {
    bindable: bool,
    allow_nic_name: bool,
    ipv6: bool,
    expect_port: bool,
    allow_dns: bool,
    allow_path: bool,
}

impl Default for IpResolverOptions {
    fn default() -> Self {
        IpResolverOptions {
            bindable: false,
            allow_nic_name: false,
            ipv6: false,
            expect_port: false,
            allow_dns: false,
            allow_path: false,
        }
    }
}

impl IpResolverOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bindable(&mut self, value: bool) -> &mut Self {
        self.bindable = value;
        self
    }

    pub fn allow_nic_name(&mut self, value: bool) -> &mut Self {
        self.allow_nic_name = value;
        self
    }

    pub fn ipv6(&mut self, value: bool) -> &mut Self {
        self.ipv6 = value;
        self
    }

    pub fn expect_port(&mut self, value: bool) -> &mut Self {
        self.expect_port = value;
        self
    }

    pub fn allow_dns(&mut self, value: bool) -> &mut Self {
        self.allow_dns = value;
        self
    }

    pub fn allow_path(&mut self, value: bool) -> &mut Self {
        self.allow_path = value;
        self
    }
}

pub struct IpResolver {
    options: IpResolverOptions,
}

impl IpResolver {
    pub fn new(opts: IpResolverOptions) -> Self {
        IpResolver { options: opts }
    }

    pub fn resolve(&self, name: &str) -> Result<IpAddrT, std::io::Error> {
        let (addr_str, port) = if self.options.expect_port {
            match name.rfind(':') {
                Some(pos) => {
                    let (addr, port_str) = name.split_at(pos);
                    let port_str = &port_str[1..];
                    let port = if port_str == "*" {
                        if self.options.bindable {
                            0
                        } else {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                "Invalid port",
                            ));
                        }
                    } else {
                        port_str.parse().map_err(|_| {
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                "Invalid port number",
                            )
                        })?
                    };
                    (addr.to_string(), port)
                }
                None => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Expected port number",
                    ))
                }
            }
        } else {
            (name.to_string(), 0)
        };

        let addr_str = if self.options.allow_path {
            match addr_str.find('/') {
                Some(pos) => &addr_str[..pos],
                None => &addr_str,
            }
        } else {
            &addr_str
        };

        // Handle IPv6 brackets
        let addr_str = if addr_str.starts_with('[') && addr_str.ends_with(']') {
            &addr_str[1..addr_str.len() - 1]
        } else {
            addr_str
        };

        // Handle zone ID/scope ID for IPv6
        let (addr_str, scope_id) = match addr_str.rfind('%') {
            Some(pos) => {
                let (addr, scope) = addr_str.split_at(pos);
                let scope = &scope[1..];
                let scope_id = if scope.chars().next().map_or(false, |c| c.is_alphabetic()) {
                    #[cfg(unix)]
                    {
                        unsafe { libc::if_nametoindex(scope.as_ptr() as *const c_char) }
                    }
                    #[cfg(not(unix))]
                    {
                        0
                    }
                } else {
                    scope.parse().unwrap_or(0)
                };
                (addr, scope_id)
            }
            None => (addr_str, 0),
        };

        // Handle wildcard address
        if self.options.bindable && addr_str == "*" {
            return Ok(IpAddrT::any(if self.options.ipv6 {
                AF_INET6 as i32
            } else {
                AF_INET as i32
            }));
        }

        // Resolve address
        let socket_addr = if self.options.allow_dns {
            (addr_str, port)
                .to_socket_addrs()?
                .next()
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Address not found"))?
        } else {
            let ip = IpAddr::from_str(addr_str).map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Invalid IP address format",
                )
            })?;
            SocketAddr::new(ip, port)
        };

        Ok(IpAddrT { inner: socket_addr })
    }
}
