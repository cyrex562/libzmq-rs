use std::collections::HashMap;
use std::mem;
use std::os::raw::c_int;

#[cfg(windows)]
use winapi::shared::ws2def::*;
#[cfg(windows)]
use winapi::um::winsock2::*;

#[cfg(not(windows))]
use libc::{fd_set, timeval, FD_CLR, FD_ISSET, FD_SET, FD_ZERO};

use crate::constants::{ZMQ_AF_UNSPEC, ZMQ_FD_CLR, ZMQ_FD_SET, ZMQ_FD_ZERO};

const FD_SETSIZE: usize = 1024;
const RETIRED_FD: i32 = -1;

#[derive(Clone)]
struct FdsSet {
    read: fd_set,
    write: fd_set,
    error: fd_set,
}

impl FdsSet {
    fn new() -> Self {
        unsafe {
            let mut read: fd_set = mem::zeroed();
            let mut write: fd_set = mem::zeroed();
            let mut error: fd_set = mem::zeroed();
            ZMQ_FD_ZERO(&mut read);
            ZMQ_FD_ZERO(&mut write);
            ZMQ_FD_ZERO(&mut error);
            FdsSet { read, write, error }
        }
    }

    fn remove_fd(&mut self, fd: i32) {
        unsafe {
            ZMQ_FD_CLR(fd as usize, &mut self.read);
            ZMQ_FD_CLR(fd as usize, &mut self.write);
            ZMQ_FD_CLR(fd as usize, &mut self.error);
        }
    }
}

struct FdEntry {
    fd: i32,
    events: Box<dyn PollEvents>,
}

struct FamilyEntry {
    fd_entries: Vec<FdEntry>,
    fds_set: FdsSet,
    has_retired: bool,
}

impl FamilyEntry {
    fn new() -> Self {
        FamilyEntry {
            fd_entries: Vec::new(),
            fds_set: FdsSet::new(),
            has_retired: false,
        }
    }
}

pub trait PollEvents {
    fn in_event(&mut self);
    fn out_event(&mut self);
}

pub struct Select {
    #[cfg(windows)]
    family_entries: HashMap<u16, FamilyEntry>,
    #[cfg(windows)]
    fd_family_cache: [(i32, u16); 8],
    #[cfg(not(windows))]
    family_entry: FamilyEntry,
    #[cfg(not(windows))]
    max_fd: i32,
}

impl Select {
    pub fn new() -> Self {
        #[cfg(windows)]
        {
            Select {
                family_entries: HashMap::new(),
                fd_family_cache: [(RETIRED_FD, 0); 8],
            }
        }
        #[cfg(not(windows))]
        {
            Select {
                family_entry: FamilyEntry::new(),
                max_fd: RETIRED_FD,
            }
        }
    }

    pub fn add_fd(&mut self, fd: i32, events: Box<dyn PollEvents>) -> i32 {
        assert!(fd != RETIRED_FD);

        let fd_entry = FdEntry { fd, events };

        #[cfg(windows)]
        {
            let family = self.get_fd_family(fd);
            assert!(family != ZMQ_AF_UNSPEC as u16);
            let family_entry = self
                .family_entries
                .entry(family)
                .or_insert_with(FamilyEntry::new);
            family_entry.fd_entries.push(fd_entry);
            unsafe {
                ZMQ_FD_SET(fd as usize, &mut family_entry.fds_set.error);
            }
        }
        #[cfg(not(windows))]
        {
            self.family_entry.fd_entries.push(fd_entry);
            unsafe {
                FD_SET(fd as usize, &mut self.family_entry.fds_set.error);
            }
            if fd > self.max_fd {
                self.max_fd = fd;
            }
        }

        fd
    }

    pub fn rm_fd(&mut self, handle: i32) {
        #[cfg(windows)]
        {
            let family = self.get_fd_family(handle);
            if let Some(family_entry) = self.family_entries.get_mut(&family) {
                if let Some(pos) = family_entry.fd_entries.iter().position(|e| e.fd == handle) {
                    family_entry.fd_entries.remove(pos);
                    family_entry.fds_set.remove_fd(handle);
                }
            }
        }
        #[cfg(not(windows))]
        {
            if let Some(pos) = self
                .family_entry
                .fd_entries
                .iter()
                .position(|e| e.fd == handle)
            {
                self.family_entry.fd_entries.remove(pos);
                self.family_entry.fds_set.remove_fd(handle);

                if handle == self.max_fd {
                    self.max_fd = self
                        .family_entry
                        .fd_entries
                        .iter()
                        .map(|e| e.fd)
                        .max()
                        .unwrap_or(RETIRED_FD);
                }
            }
        }
    }

    pub fn set_pollin(&mut self, handle: i32) {
        #[cfg(windows)]
        {
            let family = self.get_fd_family(handle);
            if let Some(family_entry) = self.family_entries.get_mut(&family) {
                unsafe {
                    ZMQ_FD_SET(handle as usize, &mut family_entry.fds_set.read);
                }
            }
        }
        #[cfg(not(windows))]
        unsafe {
            FD_SET(handle as usize, &mut self.family_entry.fds_set.read);
        }
    }

    pub fn reset_pollin(&mut self, handle: i32) {
        #[cfg(windows)]
        {
            let family = self.get_fd_family(handle);
            if let Some(family_entry) = self.family_entries.get_mut(&family) {
                unsafe {
                    ZMQ_FD_CLR(handle as usize, &mut family_entry.fds_set.read);
                }
            }
        }
        #[cfg(not(windows))]
        unsafe {
            FD_CLR(handle as usize, &mut self.family_entry.fds_set.read);
        }
    }

    #[cfg(windows)]
    fn get_fd_family(&mut self, fd: i32) -> u16 {
        // Check cache first
        if let Some(pos) = self
            .fd_family_cache
            .iter()
            .position(|&(cached_fd, _)| cached_fd == fd)
        {
            return self.fd_family_cache[pos].1;
        }

        let family = self.determine_fd_family(fd);
        // Update cache
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0, 8);
        self.fd_family_cache[idx] = (fd, family);
        family
    }

    #[cfg(windows)]
    fn determine_fd_family(&self, fd: i32) -> u16 {
        use crate::constants::{ZMQ_SOCK_DGRAM, ZMQ_SOL_SOCKET, ZMQ_SO_TYPE};

        unsafe {
            let mut addr: SOCKADDR_STORAGE = mem::zeroed();
            let mut addr_len = mem::size_of::<SOCKADDR_STORAGE>() as c_int;
            let mut sock_type = 0;
            let mut type_len = mem::size_of::<c_int>() as c_int;

            if getsockopt(
                fd as SOCKET,
                ZMQ_SOL_SOCKET,
                ZMQ_SO_TYPE,
                &mut sock_type as *mut _ as *mut i8,
                &mut type_len,
            ) == 0
            {
                if sock_type == ZMQ_SOCK_DGRAM as c_int {
                    return AF_INET as u16;
                }

                if getsockname(
                    fd as SOCKET,
                    &mut addr as *mut _ as *mut SOCKADDR,
                    &mut addr_len,
                ) != SOCKET_ERROR
                {
                    return if addr.ss_family as i32 == AF_INET6 {
                        AF_INET as u16
                    } else {
                        addr.ss_family
                    };
                }
            }
            AF_UNSPEC as u16
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockPollEvents;
    impl PollEvents for MockPollEvents {
        fn in_event(&mut self) {}
        fn out_event(&mut self) {}
    }

    #[test]
    fn test_select_creation() {
        let select = Select::new();
        #[cfg(not(windows))]
        assert_eq!(select.max_fd, RETIRED_FD);
    }

    #[test]
    fn test_add_remove_fd() {
        let mut select = Select::new();
        let events = Box::new(MockPollEvents);
        let fd = 42;

        let handle = select.add_fd(fd, events);
        assert_eq!(handle, fd);

        select.rm_fd(handle);
    }
}
