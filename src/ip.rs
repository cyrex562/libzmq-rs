#![allow(dead_code)]

use std::ffi::CString;
use std::io;
use std::mem;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::os::raw::{c_int, c_void};
#[cfg(target_os = "")]
use std::os::unix::io::{AsRawFd, RawFd};
use std::path::PathBuf;

#[cfg(unix)]
use libc::{
    self, AF_INET, AF_INET6, AF_UNIX, IPPROTO_IP, IPPROTO_IPV6, IPPROTO_TCP, IP_TOS, IPV6_TCLASS,
    O_NONBLOCK, SOCK_CLOEXEC, SOCK_STREAM, SOL_SOCKET, SO_BINDTODEVICE, SO_NOSIGPIPE, SO_PRIORITY,
    TCP_NODELAY,
};

#[cfg(windows)]
use winapi::um::winsock2::{
    self, WSADATA, WSAStartup,
};
use windows_sys::Win32::Networking::WinSock::{IPPROTO_IP, IP_TOS};

type Result<T> = std::result::Result<T, io::Error>;

pub struct Socket {
    fd: RawFd,
}

impl Socket {
    pub fn new(domain: c_int, socket_type: c_int, protocol: c_int) -> Result<Self> {
        let fd = unsafe {
            #[cfg(unix)]
            {
                libc::socket(domain, socket_type | SOCK_CLOEXEC, protocol)
            }
            #[cfg(windows)]
            {
                winsock2::socket(domain, socket_type, protocol)
            }
        };

        if fd == -1 {
            return Err(io::Error::last_os_error());
        }

        let socket = Socket { fd };
        socket.make_noninheritable()?;
        socket.set_nosigpipe()?;

        Ok(socket)
    }

    pub fn set_nonblocking(&self) -> Result<()> {
        #[cfg(unix)]
        unsafe {
            let flags = libc::fcntl(self.fd, libc::F_GETFL, 0);
            if flags == -1 {
                return Err(io::Error::last_os_error());
            }
            if libc::fcntl(self.fd, libc::F_SETFL, flags | O_NONBLOCK) == -1 {
                return Err(io::Error::last_os_error());
            }
        }

        #[cfg(windows)]
        unsafe {
            let mut nonblocking: c_int = 1;
            if winsock2::ioctlsocket(
                self.fd as _,
                winsock2::FIONBIO,
                &mut nonblocking as *mut _ as *mut _,
            ) != 0
            {
                return Err(io::Error::last_os_error());
            }
        }

        Ok(())
    }

    pub fn enable_ipv4_mapping(&self) -> Result<()> {
        #[cfg(all(unix, not(any(target_os = "openbsd", target_os = "dragonfly"))))]
        unsafe {
            let flag: c_int = 0;
            if libc::setsockopt(
                self.fd,
                IPPROTO_IPV6,
                libc::IPV6_V6ONLY,
                &flag as *const _ as *const c_void,
                mem::size_of_val(&flag) as _,
            ) == -1
            {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(())
    }

    pub fn set_ip_type_of_service(&self, tos: c_int) -> Result<()> {
        unsafe {
            if libc::setsockopt(
                self.fd,
                IPPROTO_IP,
                IP_TOS,
                &tos as *const _ as *const c_void,
                mem::size_of_val(&tos) as _,
            ) == -1
            {
                return Err(io::Error::last_os_error());
            }

            #[cfg(all(unix, target_os = "linux"))]
            {
                let _ = libc::setsockopt(
                    self.fd,
                    IPPROTO_IPV6,
                    IPV6_TCLASS,
                    &tos as *const _ as *const c_void,
                    mem::size_of_val(&tos) as _,
                );
            }
        }
        Ok(())
    }

    pub fn set_priority(&self, priority: c_int) -> Result<()> {
        #[cfg(unix)]
        unsafe {
            if libc::setsockopt(
                self.fd,
                SOL_SOCKET,
                SO_PRIORITY,
                &priority as *const _ as *const c_void,
                mem::size_of_val(&priority) as _,
            ) == -1
            {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(())
    }

    fn set_nosigpipe(&self) -> Result<()> {
        #[cfg(target_os = "macos")]
        unsafe {
            let value: c_int = 1;
            if libc::setsockopt(
                self.fd,
                SOL_SOCKET,
                SO_NOSIGPIPE,
                &value as *const _ as *const c_void,
                mem::size_of_val(&value) as _,
            ) == -1
            {
                if io::Error::last_os_error().raw_os_error() == Some(libc::EINVAL) {
                    return Err(io::Error::last_os_error());
                }
            }
        }
        Ok(())
    }

    pub fn bind_to_device(&self, device: &str) -> Result<()> {
        #[cfg(target_os = "linux")]
        unsafe {
            let device = CString::new(device)?;
            if libc::setsockopt(
                self.fd,
                SOL_SOCKET,
                SO_BINDTODEVICE,
                device.as_ptr() as *const c_void,
                device.as_bytes().len() as _,
            ) == -1
            {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(())
    }

    fn make_noninheritable(&self) -> Result<()> {
        #[cfg(unix)]
        unsafe {
            if libc::fcntl(self.fd, libc::F_SETFD, libc::FD_CLOEXEC) == -1 {
                return Err(io::Error::last_os_error());
            }
        }

        #[cfg(windows)]
        unsafe {
            use winapi::um::handleapi::SetHandleInformation;
            use winapi::um::winbase::HANDLE_FLAG_INHERIT;

            if SetHandleInformation(
                self.fd as *mut _,
                HANDLE_FLAG_INHERIT,
                0,
            ) == 0 {
                return Err(io::Error::last_os_error());
            }
        }

        Ok(())
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        unsafe {
            #[cfg(unix)]
            libc::close(self.fd);
            #[cfg(windows)]
            winsock2::closesocket(self.fd as _);
        }
    }
}

pub fn initialize_network() -> Result<()> {
    #[cfg(windows)]
    unsafe {
        let mut wsa_data: WSADATA = mem::zeroed();
        if WSAStartup(0x202, &mut wsa_data) != 0 {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}

pub fn shutdown_network() -> Result<()> {
    #[cfg(windows)]
    unsafe {
        if winsock2::WSACleanup() != 0 {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}

#[cfg(unix)]
pub fn create_ipc_wildcard_address() -> Result<(PathBuf, PathBuf)> {
    use std::env;
    use std::fs;

    let tmp_dir = env::var_os("TMPDIR")
        .or_else(|| env::var_os("TEMPDIR"))
        .or_else(|| env::var_os("TMP"))
        .unwrap_or_else(|| "/tmp".into());

    let path = PathBuf::from(tmp_dir).join("zmq-XXXXXX");
    let path_str = path.to_string_lossy().into_owned();

    unsafe {
        let mut template = CString::new(path_str)?.into_bytes_with_nul();
        if libc::mkdtemp(template.as_mut_ptr() as *mut _).is_null() {
            return Err(io::Error::last_os_error());
        }
        let dir_path = PathBuf::from(String::from_utf8(template)?);
        let socket_path = dir_path.join("socket");
        Ok((dir_path, socket_path))
    }
}

#[cfg(test)]
mod tests {
    use winapi::um::winsock2::SOCK_STREAM;
    use windows_sys::Win32::Networking::WinSock::AF_INET;
    use super::*;

    #[test]
    fn test_socket_creation() {
        let socket = Socket::new(AF_INET as c_int, SOCK_STREAM, 0).unwrap();
        assert!(socket.fd > 0);
    }

    #[test]
    fn test_network_initialization() {
        assert!(initialize_network().is_ok());
        assert!(shutdown_network().is_ok());
    }
}
