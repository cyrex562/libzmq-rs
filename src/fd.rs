#![allow(dead_code)]

// Platform-specific file descriptor type
#[cfg(windows)]
pub type FdT = usize;

#[cfg(not(windows))]
pub type FdT = i32;

// Platform-specific retired_fd constant
#[cfg(windows)]
pub const RETIRED_FD: FdT = usize::MAX;

#[cfg(not(windows))]
pub const RETIRED_FD: FdT = -1;
