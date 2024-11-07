use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct WsAddress {
    address: SocketAddr,
    host: String,
    path: String,
}

impl WsAddress {
    pub fn new() -> Self {
        WsAddress {
            address: SocketAddr::from(([0, 0, 0, 0], 0)),
            host: String::new(),
            path: String::new(),
        }
    }

    pub fn from_socket_addr(addr: SocketAddr) -> Self {
        let host = match addr.ip() {
            IpAddr::V4(ip) => ip.to_string(),
            IpAddr::V6(ip) => format!("[{}]", ip.to_string()),
        };

        WsAddress {
            address: addr,
            host,
            path: String::from("/"),
        }
    }

    pub fn resolve(&mut self, name: &str, local: bool, ipv6: bool) -> Result<(), String> {
        // Parse host and path
        let parts: Vec<&str> = name.rsplitn(2, ':').collect();
        if parts.len() != 2 {
            return Err("Invalid address format".to_string());
        }

        let host_and_path = parts[1];
        let port = parts[0].parse::<u16>().map_err(|e| e.to_string())?;

        let (host, path) = if let Some(path_idx) = host_and_path.find('/') {
            let (h, p) = host_and_path.split_at(path_idx);
            (h, p.to_string())
        } else {
            (host_and_path, "/".to_string())
        };

        // Store path
        self.path = path;
        self.host = host.to_string();

        // Resolve address
        let ip = if local {
            if host == "*" {
                if ipv6 {
                    "::".parse().unwrap()
                } else {
                    "0.0.0.0".parse().unwrap()
                }
            } else {
                IpAddr::from_str(host).map_err(|e| e.to_string())?
            }
        } else {
            // In a real implementation, this would do DNS resolution
            IpAddr::from_str(host).map_err(|e| e.to_string())?
        };

        self.address = SocketAddr::new(ip, port);
        Ok(())
    }

    pub fn to_string(&self) -> String {
        format!("ws://{}:{}{}", self.host, self.address.port(), self.path)
    }

    pub fn socket_addr(&self) -> SocketAddr {
        self.address
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn family(&self) -> i32 {
        match self.address {
            SocketAddr::V4(_) => 2, // AF_INET
            SocketAddr::V6(_) => 10, // AF_INET6
        }
    }
}

impl Default for WsAddress {
    fn default() -> Self {
        Self::new()
    }
}
