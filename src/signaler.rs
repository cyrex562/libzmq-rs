use std::io::{self, Read, Write};
use std::os::unix::io::{AsRawFd, RawFd};
use std::time::Duration;

#[cfg(unix)]
use nix::sys::socket::{self, socketpair, AddressFamily, SockFlag, SockType};
#[cfg(unix)]
use nix::unistd::{close, getpid, Pid};

#[cfg(windows)]
use windows_sys::Win32::Networking::WinSock;

pub const RETIRED_FD: RawFd = -1;

pub struct Signaler {
    reader: RawFd,
    writer: RawFd,
    #[cfg(feature = "fork")]
    pid: Pid,
}

impl Signaler {
    pub fn new() -> io::Result<Self> {
        #[cfg(unix)]
        {
            let (reader, writer) = Self::make_fdpair()?;
            Self::unblock_socket(reader)?;
            Self::unblock_socket(writer)?;
            
            Ok(Self {
                reader,
                writer,
                #[cfg(feature = "fork")]
                pid: getpid(),
            })
        }

        #[cfg(windows)]
        {
            // Windows-specific implementation would go here
            unimplemented!("Windows support not implemented")
        }
    }

    pub fn get_fd(&self) -> RawFd {
        self.reader
    }

    pub fn send(&self) -> io::Result<()> {
        #[cfg(feature = "fork")]
        if self.pid != getpid() {
            return Ok(());
        }

        #[cfg(target_os = "linux")]
        {
            let inc: u64 = 1;
            let bytes_written = unsafe {
                libc::write(
                    self.writer,
                    &inc as *const u64 as *const libc::c_void,
                    std::mem::size_of::<u64>(),
                )
            };
            if bytes_written != std::mem::size_of::<u64>() as isize {
                return Err(io::Error::last_os_error());
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            let dummy: u8 = 0;
            loop {
                match socket::send(self.writer, &[dummy], socket::MsgFlags::empty()) {
                    Ok(_) => break,
                    Err(nix::errno::Errno::EINTR) => continue,
                    Err(e) => return Err(e.into()),
                }
            }
        }

        Ok(())
    }

    pub fn wait(&self, timeout: Option<Duration>) -> io::Result<()> {
        #[cfg(feature = "fork")]
        if self.pid != getpid() {
            return Err(io::Error::from_raw_os_error(libc::EINTR));
        }

        let mut pfd = libc::pollfd {
            fd: self.reader,
            events: libc::POLLIN,
            revents: 0,
        };

        let timeout_ms = timeout.map(|t| t.as_millis() as i32).unwrap_or(-1);

        let rc = unsafe { libc::poll(&mut pfd, 1, timeout_ms) };

        match rc {
            -1 => Err(io::Error::last_os_error()),
            0 => Err(io::Error::from(io::ErrorKind::WouldBlock)),
            _ => Ok(()),
        }
    }

    pub fn recv(&self) -> io::Result<()> {
        #[cfg(target_os = "linux")]
        {
            let mut dummy: u64 = 0;
            let bytes_read = unsafe {
                libc::read(
                    self.reader,
                    &mut dummy as *mut u64 as *mut libc::c_void,
                    std::mem::size_of::<u64>(),
                )
            };
            if bytes_read != std::mem::size_of::<u64>() as isize {
                return Err(io::Error::last_os_error());
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            let mut dummy = [0u8; 1];
            match socket::recv(self.reader, &mut dummy, socket::MsgFlags::empty()) {
                Ok(_) => Ok(()),
                Err(e) => Err(e.into()),
            }
        }

        Ok(())
    }

    pub fn valid(&self) -> bool {
        self.writer != RETIRED_FD
    }

    #[cfg(feature = "fork")]
    pub fn forked(&mut self) -> io::Result<()> {
        close(self.reader)?;
        close(self.writer)?;
        let (reader, writer) = Self::make_fdpair()?;
        self.reader = reader;
        self.writer = writer;
        Ok(())
    }

    #[cfg(unix)]
    fn make_fdpair() -> io::Result<(RawFd, RawFd)> {
        let (fd1, fd2) = socketpair(
            AddressFamily::Unix,
            SockType::Stream,
            None,
            SockFlag::empty(),
        )?;
        Ok((fd1, fd2))
    }

    #[cfg(unix)]
    fn unblock_socket(fd: RawFd) -> io::Result<()> {
        use nix::fcntl::{fcntl, FcntlArg, OFlag};
        let flags = fcntl(fd, FcntlArg::F_GETFL)?;
        let flags = OFlag::from_bits_truncate(flags) | OFlag::O_NONBLOCK;
        fcntl(fd, FcntlArg::F_SETFL(flags))?;
        Ok(())
    }
}

impl Drop for Signaler {
    fn drop(&mut self) {
        if self.writer != RETIRED_FD {
            let _ = unsafe { libc::close(self.writer) };
        }
        if self.reader != RETIRED_FD {
            let _ = unsafe { libc::close(self.reader) };
        }
    }
}
