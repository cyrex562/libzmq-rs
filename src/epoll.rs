use libc::{
    epoll_create1, epoll_ctl, epoll_wait, EPOLLERR, EPOLLHUP, EPOLLIN, EPOLLOUT, EPOLL_CLOEXEC,
};
use libc::{EPOLL_CTL_ADD, EPOLL_CTL_DEL, EPOLL_CTL_MOD};
use std::collections::VecDeque;

use crate::wepoll::EPOLL_CTL_MOD;
use std::os::unix::io::{AsRawFd, RawFd};

const MAX_IO_EVENTS: usize = 128;
const RETIRED_FD: RawFd = -1;

pub trait PollEvents {
    fn in_event(&mut self);
    fn out_event(&mut self);
}

struct PollEntry {
    fd: RawFd,
    events: Box<dyn PollEvents>,
    ev: libc::epoll_event,
}

pub struct Epoll {
    epoll_fd: RawFd,
    retired: VecDeque<Box<PollEntry>>,
    load: i32,
}

impl Epoll {
    pub fn new() -> std::io::Result<Self> {
        let epoll_fd = unsafe { epoll_create1(EPOLL_CLOEXEC) };
        if epoll_fd < 0 {
            return Err(std::io::Error::last_os_error());
        }

        Ok(Epoll {
            epoll_fd,
            retired: VecDeque::new(),
            load: 0,
        })
    }

    pub fn add_fd(&mut self, fd: RawFd, events: Box<dyn PollEvents>) -> Box<PollEntry> {
        let mut ev = libc::epoll_event { events: 0, u64: 0 };

        let mut entry = Box::new(PollEntry { fd, events, ev });

        let ptr = &*entry as *const PollEntry as u64;
        entry.ev.u64 = ptr;

        unsafe {
            epoll_ctl(self.epoll_fd, EPOLL_CTL_ADD, fd, &mut entry.ev as *mut _);
        }

        self.load += 1;
        entry
    }

    pub fn rm_fd(&mut self, entry: &mut PollEntry) {
        unsafe {
            epoll_ctl(
                self.epoll_fd,
                EPOLL_CTL_DEL,
                entry.fd,
                &mut entry.ev as *mut _,
            );
        }
        entry.fd = RETIRED_FD;
        self.load -= 1;
    }

    pub fn set_pollin(&mut self, entry: &mut PollEntry) {
        entry.ev.events |= EPOLLIN as u32;
        unsafe {
            epoll_ctl(
                self.epoll_fd,
                EPOLL_CTL_MOD,
                entry.fd,
                &mut entry.ev as *mut _,
            );
        }
    }

    pub fn reset_pollin(&mut self, entry: &mut PollEntry) {
        entry.ev.events &= !(EPOLLIN as u32);
        unsafe {
            epoll_ctl(
                self.epoll_fd,
                EPOLL_CTL_MOD,
                entry.fd,
                &mut entry.ev as *mut _,
            );
        }
    }

    pub fn set_pollout(&mut self, entry: &mut PollEntry) {
        entry.ev.events |= EPOLLOUT as u32;
        unsafe {
            epoll_ctl(
                self.epoll_fd,
                EPOLL_CTL_MOD,
                entry.fd,
                &mut entry.ev as *mut _,
            );
        }
    }

    pub fn reset_pollout(&mut self, entry: &mut PollEntry) {
        entry.ev.events &= !(EPOLLOUT as u32);
        unsafe {
            epoll_ctl(
                self.epoll_fd,
                EPOLL_CTL_MOD,
                entry.fd,
                &mut entry.ev as *mut _,
            );
        }
    }

    pub fn poll(&mut self, timeout: i32) -> std::io::Result<()> {
        let mut events = vec![libc::epoll_event { events: 0, u64: 0 }; MAX_IO_EVENTS];

        let n = unsafe {
            epoll_wait(
                self.epoll_fd,
                events.as_mut_ptr(),
                MAX_IO_EVENTS as i32,
                timeout,
            )
        };

        if n < 0 {
            return Err(std::io::Error::last_os_error());
        }

        for i in 0..n as usize {
            let ptr = events[i].u64 as *mut PollEntry;
            if ptr.is_null() {
                continue;
            }

            let entry = unsafe { &mut *ptr };
            if entry.fd == RETIRED_FD {
                continue;
            }

            if (events[i].events & (EPOLLERR as u32 | EPOLLHUP as u32)) != 0 {
                entry.events.in_event();
            }
            if entry.fd == RETIRED_FD {
                continue;
            }
            if (events[i].events & EPOLLOUT as u32) != 0 {
                entry.events.out_event();
            }
            if entry.fd == RETIRED_FD {
                continue;
            }
            if (events[i].events & EPOLLIN as u32) != 0 {
                entry.events.in_event();
            }
        }

        Ok(())
    }
}

impl Drop for Epoll {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.epoll_fd);
        }
    }
}

#[cfg(test)]
mod tests {
    // Add tests here
}
