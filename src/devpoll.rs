use std::fs::OpenOptions;
use std::io::{Error, Result};
use std::os::unix::io::{AsRawFd, RawFd};
use std::time::Duration;

// Constants
const MAX_IO_EVENTS: usize = 1024;

pub type Handle = RawFd;

// Trait for poll events (equivalent to i_poll_events in C++)
pub trait PollEvents {
    fn in_event(&mut self);
    fn out_event(&mut self);
}

#[derive(Default)]
struct FdEntry {
    events: i16,
    reactor: Option<Box<dyn PollEvents>>,
    valid: bool,
    accepted: bool,
}

pub struct DevPoll {
    devpoll_fd: RawFd,
    fd_table: Vec<FdEntry>,
    pending_list: Vec<RawFd>,
}

impl DevPoll {
    pub fn new() -> Result<Self> {
        let devpoll_fd = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/poll")?
            .as_raw_fd();

        Ok(DevPoll {
            devpoll_fd,
            fd_table: Vec::new(),
            pending_list: Vec::new(),
        })
    }

    fn devpoll_ctl(&self, fd: RawFd, events: i16) -> Result<()> {
        let pfd = libc::pollfd {
            fd,
            events,
            revents: 0,
        };

        // Safe because we're writing a valid pollfd struct
        let res = unsafe {
            libc::write(
                self.devpoll_fd,
                &pfd as *const libc::pollfd as *const libc::c_void,
                std::mem::size_of::<libc::pollfd>(),
            )
        };

        if res == std::mem::size_of::<libc::pollfd>() as isize {
            Ok(())
        } else {
            Err(Error::last_os_error())
        }
    }

    pub fn add_fd(&mut self, fd: RawFd, reactor: Box<dyn PollEvents>) -> Result<Handle> {
        // Resize fd_table if necessary
        if self.fd_table.len() <= fd as usize {
            self.fd_table.resize_with(fd as usize + 1, Default::default);
        }

        let entry = &mut self.fd_table[fd as usize];
        assert!(!entry.valid);

        entry.events = 0;
        entry.reactor = Some(reactor);
        entry.valid = true;
        entry.accepted = false;

        self.devpoll_ctl(fd, 0)?;
        self.pending_list.push(fd);

        Ok(fd)
    }

    pub fn rm_fd(&mut self, handle: Handle) -> Result<()> {
        let entry = &mut self.fd_table[handle as usize];
        assert!(entry.valid);

        self.devpoll_ctl(handle, libc::POLLREMOVE)?;
        entry.valid = false;
        Ok(())
    }

    pub fn set_pollin(&mut self, handle: Handle) -> Result<()> {
        self.devpoll_ctl(handle, libc::POLLREMOVE)?;
        let entry = &mut self.fd_table[handle as usize];
        entry.events |= libc::POLLIN as i16;
        self.devpoll_ctl(handle, entry.events)
    }

    pub fn reset_pollin(&mut self, handle: Handle) -> Result<()> {
        self.devpoll_ctl(handle, libc::POLLREMOVE)?;
        let entry = &mut self.fd_table[handle as usize];
        entry.events &= !(libc::POLLIN as i16);
        self.devpoll_ctl(handle, entry.events)
    }

    pub fn set_pollout(&mut self, handle: Handle) -> Result<()> {
        self.devpoll_ctl(handle, libc::POLLREMOVE)?;
        let entry = &mut self.fd_table[handle as usize];
        entry.events |= libc::POLLOUT as i16;
        self.devpoll_ctl(handle, entry.events)
    }

    pub fn reset_pollout(&mut self, handle: Handle) -> Result<()> {
        self.devpoll_ctl(handle, libc::POLLREMOVE)?;
        let entry = &mut self.fd_table[handle as usize];
        entry.events &= !(libc::POLLOUT as i16);
        self.devpoll_ctl(handle, entry.events)
    }

    pub fn poll(&mut self, timeout: Option<Duration>) -> Result<()> {
        let mut ev_buf = vec![
            libc::pollfd {
                fd: -1,
                events: 0,
                revents: 0,
            };
            MAX_IO_EVENTS
        ];

        // Accept all pending FDs
        for &fd in &self.pending_list {
            if let Some(entry) = self.fd_table.get_mut(fd as usize) {
                entry.accepted = true;
            }
        }
        self.pending_list.clear();

        let timeout_ms = timeout.map_or(-1, |t| t.as_millis() as i32);

        let poll_req = libc::dvpoll {
            dp_fds: ev_buf.as_mut_ptr(),
            dp_nfds: MAX_IO_EVENTS as i32,
            dp_timeout: timeout_ms,
        };

        // Safe because we're using properly initialized structures
        let n = unsafe {
            libc::ioctl(
                self.devpoll_fd,
                libc::DP_POLL,
                &poll_req as *const libc::dvpoll,
            )
        };

        if n == -1 {
            return Err(Error::last_os_error());
        }

        for i in 0..n {
            let ev = unsafe { ev_buf.get_unchecked(i as usize) };
            if let Some(entry) = self.fd_table.get_mut(ev.fd as usize) {
                if !entry.valid || !entry.accepted {
                    continue;
                }

                if let Some(reactor) = &mut entry.reactor {
                    if ev.revents & (libc::POLLERR | libc::POLLHUP) != 0 {
                        reactor.in_event();
                    }
                    if ev.revents & libc::POLLOUT != 0 {
                        reactor.out_event();
                    }
                    if ev.revents & libc::POLLIN != 0 {
                        reactor.in_event();
                    }
                }
            }
        }

        Ok(())
    }
}

impl Drop for DevPoll {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.devpoll_fd);
        }
    }
}
