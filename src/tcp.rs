use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::os::unix::io::{AsRawFd, RawFd};
#[cfg(target_os = "windows")]
use std::os::windows::io::{AsRawSocket, RawSocket};

#[derive(Debug)]
pub struct TcpOptions {
    pub sndbuf: i32,
    pub rcvbuf: i32,
    pub tos: u8,
    pub priority: i32,
    pub loopback_fastpath: bool,
    pub busy_poll: i32,
    pub bound_device: String,
    pub ipv6: bool,
}

impl Default for TcpOptions {
    fn default() -> Self {
        TcpOptions {
            sndbuf: -1,
            rcvbuf: -1,
            tos: 0,
            priority: 0,
            loopback_fastpath: false,
            busy_poll: 0,
            bound_device: String::new(),
            ipv6: false,
        }
    }
}

pub fn tune_tcp_socket(stream: &TcpStream) -> std::io::Result<()> {
    // Disable Nagle's algorithm
    stream.set_nodelay(true)?;
    Ok(())
}

pub fn set_tcp_send_buffer(stream: &TcpStream, size: usize) -> std::io::Result<()> {
    stream.set_send_buffer_size(size)
}

pub fn set_tcp_receive_buffer(stream: &TcpStream, size: usize) -> std::io::Result<()> {
    stream.set_recv_buffer_size(size)
}

pub fn tune_tcp_keepalives(
    stream: &TcpStream,
    keepalive: bool,
    keepalive_cnt: u32,
    keepalive_idle: u32,
    keepalive_intvl: u32,
) -> std::io::Result<()> {
    stream.set_keepalive(Some(std::time::Duration::from_secs(keepalive_idle as u64)))?;
    
    #[cfg(target_family = "unix")]
    {
        use std::os::unix::io::AsRawFd;
        let fd = stream.as_raw_fd();
        
        #[cfg(target_os = "linux")]
        unsafe {
            let optval: libc::c_int = keepalive_cnt as libc::c_int;
            libc::setsockopt(
                fd,
                libc::IPPROTO_TCP,
                libc::TCP_KEEPCNT,
                &optval as *const _ as *const libc::c_void,
                std::mem::size_of_val(&optval) as libc::socklen_t,
            );

            let optval: libc::c_int = keepalive_intvl as libc::c_int;
            libc::setsockopt(
                fd,
                libc::IPPROTO_TCP,
                libc::TCP_KEEPINTVL,
                &optval as *const _ as *const libc::c_void,
                std::mem::size_of_val(&optval) as libc::socklen_t,
            );
        }
    }

    Ok(())
}

pub fn tcp_write(stream: &mut TcpStream, data: &[u8]) -> std::io::Result<usize> {
    stream.write(data)
}

pub fn tcp_read(stream: &mut TcpStream, data: &mut [u8]) -> std::io::Result<usize> {
    stream.read(data)
}

pub fn tcp_tune_loopback_fast_path(stream: &TcpStream) {
    #[cfg(target_os = "windows")]
    {
        // Windows-specific loopback fastpath implementation would go here
        // Using raw WinAPI calls if needed
    }
}

pub fn tune_tcp_busy_poll(stream: &TcpStream, busy_poll: i32) {
    #[cfg(target_os = "linux")]
    unsafe {
        let fd = stream.as_raw_fd();
        libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_BUSY_POLL,
            &busy_poll as *const _ as *const libc::c_void,
            std::mem::size_of_val(&busy_poll) as libc::socklen_t,
        );
    }
}

pub fn tcp_open_socket(
    address: &str,
    options: &TcpOptions,
    local: bool,
    fallback_to_ipv4: bool,
) -> std::io::Result<TcpStream> {
    let addr = address.to_socket_addrs()?.next().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid address")
    })?;

    let stream = TcpStream::connect(addr)?;

    if options.sndbuf >= 0 {
        set_tcp_send_buffer(&stream, options.sndbuf as usize)?;
    }
    if options.rcvbuf >= 0 {
        set_tcp_receive_buffer(&stream, options.rcvbuf as usize)?;
    }

    if options.loopback_fastpath {
        tcp_tune_loopback_fast_path(&stream);
    }

    if options.busy_poll > 0 {
        tune_tcp_busy_poll(&stream, options.busy_poll);
    }

    Ok(stream)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_options_default() {
        let opts = TcpOptions::default();
        assert_eq!(opts.sndbuf, -1);
        assert_eq!(opts.rcvbuf, -1);
        assert_eq!(opts.tos, 0);
        assert_eq!(opts.priority, 0);
        assert_eq!(opts.loopback_fastpath, false);
        assert_eq!(opts.busy_poll, 0);
        assert!(opts.bound_device.is_empty());
        assert_eq!(opts.ipv6, false);
    }
}
