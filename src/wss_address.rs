/* SPDX-License-Identifier: MPL-2.0 */

use std::net::SocketAddr;
use std::fmt;

// Assuming ws_address is already implemented in Rust
use super::ws_address::WsAddress;

pub struct WssAddress {
    inner: WsAddress,
}

impl WssAddress {
    pub fn new() -> Self {
        Self {
            inner: WsAddress::new()
        }
    }

    pub fn from_sockaddr(sa: &SocketAddr) -> Self {
        Self {
            inner: WsAddress::from_sockaddr(sa)
        }
    }
}

impl fmt::Display for WssAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "wss://{}:{}{}", 
            self.inner.host(), 
            self.inner.port(), 
            self.inner.path()
        )
    }
}

impl Default for WssAddress {
    fn default() -> Self {
        Self::new()
    }
}
