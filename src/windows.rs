#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows_sys::Win32::Networking::WinSock;
use windows_sys::Win32::System::WindowsProgramming;
use windows_sys::Win32::Foundation::*;

// Original version targeting constants
pub const WIN32_WINNT_MIN: u32 = 0x0600; // Windows Vista
pub const WIN32_MINGW_XP_MIN: u32 = 0x0501; // Windows XP minimum for MinGW

#[cfg(feature = "mingw32")]
pub const SIO_KEEPALIVE_VALS: u32 = WindowsProgramming::IOC_VENDOR + 4;

#[repr(C)]
pub struct tcp_keepalive {
    pub onoff: u32,
    pub keepalivetime: u32,
    pub keepaliveinterval: u32,
}

// Convert UTF-8 to UTF-16 for Windows APIs
pub fn utf8_to_utf16<P: AsRef<Path>>(path: P) -> Vec<u16> {
    path.as_ref()
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

// Unlink (delete) file with UTF-8 path
pub fn unlink_utf8<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    let wide_path = utf8_to_utf16(path);
    let result = unsafe {
        windows_sys::Win32::Storage::FileSystem::DeleteFileW(wide_path.as_ptr())
    };
    if result == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

// Remove directory with UTF-8 path
pub fn rmdir_utf8<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    let wide_path = utf8_to_utf16(path);
    let result = unsafe {
        windows_sys::Win32::Storage::FileSystem::RemoveDirectoryW(wide_path.as_ptr())
    };
    if result == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(feature = "ipc")]
#[repr(C)]
pub struct sockaddr_un {
    pub sun_family: u16, // AF_UNIX
    pub sun_path: [u8; 108],
}

// Poll wrapper for Windows
#[cfg(any(feature = "io-poll", feature = "poll-based"))]
pub fn poll(fds: &mut [WinSock::WSAPOLLFD], timeout: i32) -> std::io::Result<i32> {
    let result = unsafe {
        WinSock::WSAPoll(
            fds.as_mut_ptr(),
            fds.len() as u32,
            timeout
        )
    };
    if result == SOCKET_ERROR as i32 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(result)
    }
}
