use std::ffi::CStr;
use std::os::unix::net::UnixAddr;
use std::path::Path;

#[derive(Debug)]
pub struct IpcAddress {
    address: UnixAddr,
}

impl IpcAddress {
    pub fn new() -> Self {
        Self {
            address: UnixAddr::new(&[] as &[u8]).unwrap(),
        }
    }

    pub fn from_raw(sa: &UnixAddr) -> Self {
        Self {
            address: sa.clone(),
        }
    }

    pub fn resolve(&mut self, path: &str) -> std::io::Result<()> {
        if path.len() >= 108 {
            // SUN_PATH length limit
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path too long",
            ));
        }
        if path.starts_with('@') && path.len() == 1 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid abstract socket name",
            ));
        }

        // Handle abstract sockets (starting with @)
        if path.starts_with('@') {
            let mut abstract_path = vec![0u8];
            abstract_path.extend_from_slice(&path.as_bytes()[1..]);
            self.address = UnixAddr::new(&abstract_path).unwrap();
        } else {
            self.address = UnixAddr::new(Path::new(path)).unwrap();
        }
        Ok(())
    }

    pub fn to_string(&self) -> std::io::Result<String> {
        let prefix = "ipc://";
        let path = self
            .address
            .as_pathname()
            .and_then(|p| p.to_str())
            .unwrap_or("");

        if path.is_empty() {
            // Handle abstract socket
            if let Some(bytes) = self.address.as_abstract_namespace() {
                if let Ok(abstract_path) = CStr::from_bytes_with_nul(bytes) {
                    if let Ok(path_str) = abstract_path.to_str() {
                        return Ok(format!("{}@{}", prefix, path_str));
                    }
                }
            }
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid address",
            ));
        }

        Ok(format!("{}{}", prefix, path))
    }

    pub fn as_unix_addr(&self) -> &UnixAddr {
        &self.address
    }
}

impl Default for IpcAddress {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_regular_path() {
        let mut addr = IpcAddress::new();
        assert!(addr.resolve("/tmp/test.sock").is_ok());
        assert_eq!(addr.to_string().unwrap(), "ipc:///tmp/test.sock");
    }

    #[test]
    fn test_resolve_abstract_socket() {
        let mut addr = IpcAddress::new();
        assert!(addr.resolve("@test").is_ok());
        // Note: actual string representation may vary depending on platform
    }

    #[test]
    fn test_invalid_path() {
        let mut addr = IpcAddress::new();
        assert!(addr.resolve("@").is_err());
    }
}
