use libc::{self, c_int, c_short, intptr_t, pid_t, timespec};
use std::{io, ptr, slice};
use std::os::unix::io::RawFd;

const MAX_IO_EVENTS: usize = 32;

type Handle = *mut PollEntry;

#[derive(Debug)]
pub struct PollEntry {
    fd: RawFd,
    flag_pollin: bool,
    flag_pollout: bool,
    reactor: Box<dyn IPollEvents>,
}

pub trait IPollEvents {
    fn in_event(&mut self);
    fn out_event(&mut self);
}

pub struct Kqueue {
    kqueue_fd: RawFd,
    retired: Vec<*mut PollEntry>,
    #[cfg(feature = "fork")]
    pid: pid_t,
}

impl Kqueue {
    pub fn new() -> io::Result<Self> {
        let kqueue_fd = unsafe { libc::kqueue() };
        if kqueue_fd == -1 {
            return Err(io::Error::last_os_error());
        }

        Ok(Kqueue {
            kqueue_fd,
            retired: Vec::new(),
            #[cfg(feature = "fork")]
            pid: unsafe { libc::getpid() },
        })
    }

    fn kevent_add(&self, fd: RawFd, filter: c_short, udata: *mut PollEntry) -> io::Result<()> {
        let mut ev: libc::kevent = unsafe { std::mem::zeroed() };
        unsafe {
            libc::EV_SET(
                &mut ev,
                fd as usize,
                filter as u16,
                libc::EV_ADD as u16,
                0,
                0,
                udata as *mut _,
            );
            
            if libc::kevent(self.kqueue_fd, &ev, 1, ptr::null_mut(), 0, ptr::null()) == -1 {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(())
    }

    fn kevent_delete(&self, fd: RawFd, filter: c_short) -> io::Result<()> {
        let mut ev: libc::kevent = unsafe { std::mem::zeroed() };
        unsafe {
            libc::EV_SET(
                &mut ev,
                fd as usize,
                filter as u16,
                libc::EV_DELETE as u16,
                0,
                0,
                ptr::null_mut(),
            );
            
            if libc::kevent(self.kqueue_fd, &ev, 1, ptr::null_mut(), 0, ptr::null()) == -1 {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(())
    }

    pub fn add_fd(&mut self, fd: RawFd, reactor: Box<dyn IPollEvents>) -> Handle {
        let entry = Box::new(PollEntry {
            fd,
            flag_pollin: false,
            flag_pollout: false,
            reactor,
        });
        Box::into_raw(entry)
    }

    pub fn rm_fd(&mut self, handle: Handle) -> io::Result<()> {
        let entry = unsafe { &*handle };
        
        if entry.flag_pollin {
            self.kevent_delete(entry.fd, libc::EVFILT_READ)?;
        }
        if entry.flag_pollout {
            self.kevent_delete(entry.fd, libc::EVFILT_WRITE)?;
        }
        
        self.retired.push(handle);
        Ok(())
    }

    pub fn set_pollin(&mut self, handle: Handle) -> io::Result<()> {
        let entry = unsafe { &mut *handle };
        if !entry.flag_pollin {
            entry.flag_pollin = true;
            self.kevent_add(entry.fd, libc::EVFILT_READ, handle)?;
        }
        Ok(())
    }

    pub fn reset_pollin(&mut self, handle: Handle) -> io::Result<()> {
        let entry = unsafe { &mut *handle };
        if entry.flag_pollin {
            entry.flag_pollin = false;
            self.kevent_delete(entry.fd, libc::EVFILT_READ)?;
        }
        Ok(())
    }

    pub fn set_pollout(&mut self, handle: Handle) -> io::Result<()> {
        let entry = unsafe { &mut *handle };
        if !entry.flag_pollout {
            entry.flag_pollout = true;
            self.kevent_add(entry.fd, libc::EVFILT_WRITE, handle)?;
        }
        Ok(())
    }

    pub fn reset_pollout(&mut self, handle: Handle) -> io::Result<()> {
        let entry = unsafe { &mut *handle };
        if entry.flag_pollout {
            entry.flag_pollout = false;
            self.kevent_delete(entry.fd, libc::EVFILT_WRITE)?;
        }
        Ok(())
    }

    pub fn poll(&mut self, timeout: i32) -> io::Result<()> {
        let mut events: Vec<libc::kevent> = Vec::with_capacity(MAX_IO_EVENTS);
        let timeout = timespec {
            tv_sec: (timeout / 1000) as i64,
            tv_nsec: ((timeout % 1000) * 1000000) as i64,
        };

        unsafe {
            let n = libc::kevent(
                self.kqueue_fd,
                ptr::null(),
                0,
                events.as_mut_ptr(),
                MAX_IO_EVENTS as c_int,
                &timeout,
            );

            if n == -1 {
                return Err(io::Error::last_os_error());
            }

            events.set_len(n as usize);

            #[cfg(feature = "fork")]
            if self.pid != libc::getpid() {
                return Ok(());
            }

            for ev in events.iter() {
                let entry = &mut *(ev.udata as *mut PollEntry);
                
                if ev.flags & libc::EV_EOF as u16 != 0 {
                    entry.reactor.in_event();
                }
                if ev.filter == libc::EVFILT_WRITE {
                    entry.reactor.out_event();
                }
                if ev.filter == libc::EVFILT_READ {
                    entry.reactor.in_event();
                }
            }

            // Clean up retired entries
            for handle in self.retired.drain(..) {
                drop(Box::from_raw(handle));
            }
        }

        Ok(())
    }
}

impl Drop for Kqueue {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.kqueue_fd);
            for handle in self.retired.drain(..) {
                drop(Box::from_raw(handle));
            }
        }
    }
}

unsafe impl Send for Kqueue {}
unsafe impl Sync for Kqueue {}
