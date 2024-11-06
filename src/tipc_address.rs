#[cfg(feature = "tipc")]
use std::{mem, str::FromStr};

#[cfg(feature = "tipc")]
#[repr(C)]
pub struct sockaddr_tipc {
    family: u16,
    addrtype: u32,
    scope: u32,
    addr: tipc_addr_union,
}

#[cfg(feature = "tipc")]
#[repr(C)]
union tipc_addr_union {
    nameseq: tipc_name_seq,
    name: tipc_name,
    id: tipc_sock_addr,
}

#[cfg(feature = "tipc")]
#[repr(C)]
struct tipc_name_seq {
    typ: u32,
    lower: u32,
    upper: u32,
}

#[cfg(feature = "tipc")]
#[repr(C)]
struct tipc_name {
    name: tipc_name_seq,
    domain: u32,
}

#[cfg(feature = "tipc")]
#[repr(C)]
struct tipc_sock_addr {
    node: u32,
    ref_: u32,
}

#[cfg(feature = "tipc")]
pub struct TipcAddress {
    random: bool,
    address: sockaddr_tipc,
}

#[cfg(feature = "tipc")]
impl TipcAddress {
    pub fn new() -> Self {
        Self {
            random: false,
            address: unsafe { mem::zeroed() },
        }
    }

    pub fn from_sockaddr(sa: &sockaddr_tipc) -> Self {
        let mut addr = Self::new();
        if sa.family == libc::AF_TIPC as u16 {
            addr.address = *sa;
        }
        addr
    }

    pub fn set_random(&mut self) {
        self.random = true;
    }

    pub fn is_random(&self) -> bool {
        self.random
    }

    pub fn is_service(&self) -> bool {
        unsafe { self.address.addrtype != libc::TIPC_ADDR_ID }
    }

    pub fn resolve(&mut self, name: &str) -> Result<(), std::io::Error> {
        if name == "<*>" {
            self.set_random();
            self.address.family = libc::AF_TIPC as u16;
            self.address.addrtype = libc::TIPC_ADDR_ID;
            unsafe {
                self.address.addr.id.node = 0;
                self.address.addr.id.ref_ = 0;
            }
            self.address.scope = 0;
            return Ok(());
        }

        // Parse "{type,lower,upper}" format
        if let Some(caps) = name.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
            let parts: Vec<&str> = caps.split(',').collect();
            if parts.len() == 3 {
                let typ: u32 = parts[0].trim().parse().map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidInput))?;
                let lower: u32 = parts[1].trim().parse().map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidInput))?;
                let upper: u32 = parts[2].trim().parse().map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidInput))?;

                if typ < TIPC_RESERVED_TYPES || upper < lower {
                    return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
                }

                self.address.family = libc::AF_TIPC as u16;
                self.address.addrtype = libc::TIPC_ADDR_NAMESEQ;
                unsafe {
                    self.address.addr.nameseq.typ = typ;
                    self.address.addr.nameseq.lower = lower;
                    self.address.addr.nameseq.upper = upper;
                }
                self.address.scope = libc::TIPC_ZONE_SCOPE;
                return Ok(());
            }
        }

        Err(std::io::Error::from(std::io::ErrorKind::InvalidInput))
    }

    pub fn to_string(&self) -> Result<String, std::io::Error> {
        if self.address.family != libc::AF_TIPC as u16 {
            return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
        }

        let addr = unsafe {
            if self.address.addrtype == libc::TIPC_ADDR_NAMESEQ {
                format!("tipc://{{type={}, lower={}, upper={}}}",
                    self.address.addr.nameseq.typ,
                    self.address.addr.nameseq.lower,
                    self.address.addr.nameseq.upper)
            } else if self.address.addrtype == libc::TIPC_ADDR_ID || self.is_random() {
                format!("tipc://<{}:{}:{}:{}>",
                    tipc_zone(self.address.addr.id.node),
                    tipc_cluster(self.address.addr.id.node),
                    tipc_node(self.address.addr.id.node),
                    self.address.addr.id.ref_)
            } else {
                return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
            }
        };

        Ok(addr)
    }

    pub fn as_sockaddr(&self) -> &sockaddr_tipc {
        &self.address
    }

    pub fn addr_len(&self) -> usize {
        mem::size_of::<sockaddr_tipc>()
    }
}

#[cfg(feature = "tipc")]
const TIPC_RESERVED_TYPES: u32 = 64;

#[cfg(feature = "tipc")]
fn tipc_addr(z: u32, c: u32, n: u32) -> u32 {
    ((z & 0xFF) << 24) | ((c & 0xFF) << 16) | (n & 0xFFFF)
}

#[cfg(feature = "tipc")]
fn tipc_zone(addr: u32) -> u32 {
    (addr >> 24) & 0xFF
}

#[cfg(feature = "tipc")]
fn tipc_cluster(addr: u32) -> u32 {
    (addr >> 16) & 0xFF
}

#[cfg(feature = "tipc")]
fn tipc_node(addr: u32) -> u32 {
    addr & 0xFFFF
}
