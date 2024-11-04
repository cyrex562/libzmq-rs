#[cfg(target_os = "windows")]
pub use windows_compat::*;

#[cfg(not(target_os = "windows"))]
pub use unix_compat::*;

#[cfg(target_os = "windows")]
mod windows_compat {
    pub fn strcasecmp(s1: &str, s2: &str) -> i32 {
        s1.to_lowercase().cmp(&s2.to_lowercase()) as i32
    }

    // Rust's split is already thread-safe, but we provide this for compatibility
    pub fn strtok_r<'a>(s: &'a str, delim: &str) -> impl Iterator<Item = &'a str> {
        s.split(delim)
    }
}

#[cfg(not(target_os = "windows"))]
mod unix_compat {
    pub fn strlcpy(dest: &mut [u8], src: &[u8]) -> usize {
        let src_len = src.len();
        if dest.is_empty() {
            return src_len;
        }

        let copy_len = dest.len().min(src_len);
        dest[..copy_len].copy_from_slice(&src[..copy_len]);
        
        if copy_len < dest.len() {
            dest[copy_len] = 0; // Null terminator
        }
        
        src_len
    }

    pub fn strcpy_s(dest: &mut [u8], src: &[u8]) -> i32 {
        if strlcpy(dest, src) >= dest.len() {
            libc::ERANGE
        } else {
            0
        }
    }
}

// Common functionality
pub fn strnlen(s: &[u8], max_len: usize) -> usize {
    s.iter()
        .take(max_len)
        .position(|&c| c == 0)
        .unwrap_or(max_len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strnlen() {
        let s = b"Hello\0World";
        assert_eq!(strnlen(s, 10), 5);
        assert_eq!(strnlen(s, 4), 4);
    }
}
