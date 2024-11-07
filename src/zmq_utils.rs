#![allow(dead_code)]

use std::{thread, time};
use std::time::{SystemTime, UNIX_EPOCH};

// Sleep function
pub fn zmq_sleep(seconds: i32) {
    thread::sleep(time::Duration::from_secs(seconds as u64));
}

// Stopwatch functionality
pub struct Stopwatch {
    start_time: u64,
}

impl Stopwatch {
    pub fn new() -> Self {
        Stopwatch {
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64,
        }
    }

    pub fn intermediate(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;
        now - self.start_time
    }
}

// Z85 encoding/decoding
const ENCODER: &[u8; 85] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";

const DECODER: [u8; 96] = [
    0xFF, 0x44, 0xFF, 0x54, 0x53, 0x52, 0x48, 0xFF, 0x4B, 0x4C, 0x46, 0x41,
    0xFF, 0x3F, 0x3E, 0x45, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
    0x08, 0x09, 0x40, 0xFF, 0x49, 0x42, 0x4A, 0x47, 0x51, 0x24, 0x25, 0x26,
    0x27, 0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F, 0x30, 0x31, 0x32,
    0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x4D,
    0xFF, 0x4E, 0x43, 0xFF, 0xFF, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10,
    0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C,
    0x1D, 0x1E, 0x1F, 0x20, 0x21, 0x22, 0x23, 0x4F, 0xFF, 0x50, 0xFF, 0xFF
];

pub fn z85_encode(data: &[u8]) -> Option<String> {
    if data.len() % 4 != 0 {
        return None;
    }

    let mut result = Vec::with_capacity(data.len() * 5 / 4);
    let mut value: u32 = 0;

    for (i, &byte) in data.iter().enumerate() {
        value = value * 256 + byte as u32;
        if (i + 1) % 4 == 0 {
            let mut divisor = 85u32.pow(4);
            while divisor > 0 {
                result.push(ENCODER[(value / divisor % 85) as usize]);
                divisor /= 85;
            }
            value = 0;
        }
    }

    String::from_utf8(result).ok()
}

pub fn z85_decode(string: &str) -> Option<Vec<u8>> {
    if string.len() % 5 != 0 || string.len() < 5 {
        return None;
    }

    let mut result = Vec::with_capacity(string.len() * 4 / 5);
    let mut value: u32 = 0;

    for (i, c) in string.bytes().enumerate() {
        let index = (c as usize).checked_sub(32)?;
        if index >= DECODER.len() {
            return None;
        }
        let decoded = DECODER[index];
        if decoded == 0xFF {
            return None;
        }
        value = value.checked_mul(85)?.checked_add(decoded as u32)?;

        if (i + 1) % 5 == 0 {
            let mut divisor = 256u32.pow(3);
            while divisor > 0 {
                result.push((value / divisor % 256) as u8);
                divisor /= 256;
            }
            value = 0;
        }
    }

    Some(result)
}

// Atomic counter
use std::sync::atomic::{AtomicI32, Ordering};

pub struct AtomicCounter {
    value: AtomicI32,
}

impl AtomicCounter {
    pub fn new() -> Self {
        AtomicCounter {
            value: AtomicI32::new(0),
        }
    }

    pub fn set(&self, value: i32) {
        self.value.store(value, Ordering::SeqCst);
    }

    pub fn inc(&self) -> i32 {
        self.value.fetch_add(1, Ordering::SeqCst)
    }

    pub fn dec(&self) -> i32 {
        let prev = self.value.fetch_sub(1, Ordering::SeqCst);
        if prev >= 1 { 1 } else { 0 }
    }

    pub fn get(&self) -> i32 {
        self.value.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_z85_codec() {
        let data = vec![0, 1, 2, 3];
        let encoded = z85_encode(&data).unwrap();
        let decoded = z85_decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn test_atomic_counter() {
        let counter = AtomicCounter::new();
        counter.set(5);
        assert_eq!(counter.get(), 5);
        assert_eq!(counter.inc(), 5);
        assert_eq!(counter.get(), 6);
        assert_eq!(counter.dec(), 1);
        assert_eq!(counter.get(), 5);
    }
}
