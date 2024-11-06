use std::os::unix::io::RawFd;
use std::vec::Vec;

#[cfg(target_os = "windows")]
compile_error!("poll is broken on Windows for the purpose of the I/O thread poller, use select instead");

// Constants
const RETIRED_FD: RawFd = -1;

// Trait for poll events (equivalent to i_poll_events)
pub trait PollEvents {
    fn in_event(&self);
    fn out_event(&self);
}

// Equivalent to fd_entry_t
struct FdEntry {
    index: RawFd,
    events: Box<dyn PollEvents>,
}

// Main poll implementation
pub struct Poll {
    fd_table: Vec<FdEntry>,
    pollset: Vec<libc::pollfd>,
    retired: bool,
    load: i32,
}

impl Poll {
    pub fn new() -> Self {
        Poll {
            fd_table: Vec::new(),
            pollset: Vec::new(),
            retired: false,
            load: 0,
        }
    }

    pub fn add_fd(&mut self, fd: RawFd, events: Box<dyn PollEvents>) -> RawFd {
        assert!(fd != RETIRED_FD);

        // Expand fd_table if needed
        if self.fd_table.len() <= fd as usize {
            let old_size = self.fd_table.len();
            self.fd_table.resize_with(fd as usize + 1, || FdEntry {
                index: RETIRED_FD,
                events: Box::new(()),
            });
        }

        let pfd = libc::pollfd {
            fd,
            events: 0,
            revents: 0,
        };
        self.pollset.push(pfd);
        
        let index = (self.pollset.len() - 1) as RawFd;
        self.fd_table[fd as usize] = FdEntry { index, events };

        // Increase load
        self.adjust_load(1);

        fd
    }

    pub fn rm_fd(&mut self, handle: RawFd) {
        let index = self.fd_table[handle as usize].index;
        assert!(index != RETIRED_FD);

        // Mark fd as unused
        self.pollset[index as usize].fd = RETIRED_FD;
        self.fd_table[handle as usize].index = RETIRED_FD;
        self.retired = true;

        // Decrease load
        self.adjust_load(-1);
    }

    pub fn set_pollin(&mut self, handle: RawFd) {
        let index = self.fd_table[handle as usize].index;
        self.pollset[index as usize].events |= libc::POLLIN;
    }

    pub fn reset_pollin(&mut self, handle: RawFd) {
        let index = self.fd_table[handle as usize].index;
        self.pollset[index as usize].events &= !libc::POLLIN;
    }

    pub fn set_pollout(&mut self, handle: RawFd) {
        let index = self.fd_table[handle as usize].index;
        self.pollset[index as usize].events |= libc::POLLOUT;
    }

    pub fn reset_pollout(&mut self, handle: RawFd) {
        let index = self.fd_table[handle as usize].index;
        self.pollset[index as usize].events &= !libc::POLLOUT;
    }

    pub fn max_fds() -> i32 {
        -1
    }

    fn cleanup_retired(&mut self) {
        if self.retired {
            let mut i = 0;
            while i < self.pollset.len() {
                if self.pollset[i].fd == RETIRED_FD {
                    self.pollset.remove(i);
                } else {
                    self.fd_table[self.pollset[i].fd as usize].index = i as RawFd;
                    i += 1;
                }
            }
            self.retired = false;
        }
    }

    pub fn poll(&mut self, timeout: i32) -> std::io::Result<()> {
        self.cleanup_retired();

        if self.pollset.is_empty() {
            assert_eq!(self.load, 0);
            return Ok(());
        }

        // Wait for events
        let rc = unsafe {
            libc::poll(
                self.pollset.as_mut_ptr(),
                self.pollset.len() as libc::nfds_t,
                timeout,
            )
        };

        if rc == -1 {
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::Interrupted {
                return Ok(());
            }
            return Err(err);
        }

        if rc == 0 {
            return Ok(());
        }

        // Handle events
        for i in 0..self.pollset.len() {
            let pollfd = &self.pollset[i];
            if pollfd.fd == RETIRED_FD {
                continue;
            }

            let events = &self.fd_table[pollfd.fd as usize].events;
            
            if (pollfd.revents & (libc::POLLERR | libc::POLLHUP)) != 0 {
                events.in_event();
            }
            if pollfd.fd == RETIRED_FD {
                continue;
            }
            if (pollfd.revents & libc::POLLOUT) != 0 {
                events.out_event();
            }
            if pollfd.fd == RETIRED_FD {
                continue;
            }
            if (pollfd.revents & libc::POLLIN) != 0 {
                events.in_event();
            }
        }

        Ok(())
    }

    fn adjust_load(&mut self, amount: i32) {
        self.load += amount;
    }
}

// Implement PollEvents for unit type as a placeholder
impl PollEvents for () {
    fn in_event(&self) {}
    fn out_event(&self) {}
}
