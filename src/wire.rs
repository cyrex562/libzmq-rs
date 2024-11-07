//! Network byte order conversion utilities

/// Put a u8 value into a mutable byte slice
#[inline]
pub fn put_u8(buffer: &mut [u8], value: u8) {
    buffer[0] = value;
}

/// Get a u8 value from a byte slice
#[inline]
pub fn get_u8(buffer: &[u8]) -> u8 {
    buffer[0]
}

/// Put a u16 value into a mutable byte slice in network byte order
#[inline]
pub fn put_u16(buffer: &mut [u8], value: u16) {
    buffer[0] = ((value >> 8) & 0xff) as u8;
    buffer[1] = (value & 0xff) as u8;
}

/// Get a u16 value from a byte slice in network byte order
#[inline]
pub fn get_u16(buffer: &[u8]) -> u16 {
    ((buffer[0] as u16) << 8) | (buffer[1] as u16)
}

/// Put a u32 value into a mutable byte slice in network byte order
#[inline]
pub fn put_u32(buffer: &mut [u8], value: u32) {
    buffer[0] = ((value >> 24) & 0xff) as u8;
    buffer[1] = ((value >> 16) & 0xff) as u8;
    buffer[2] = ((value >> 8) & 0xff) as u8;
    buffer[3] = (value & 0xff) as u8;
}

/// Get a u32 value from a byte slice in network byte order
#[inline]
pub fn get_u32(buffer: &[u8]) -> u32 {
    ((buffer[0] as u32) << 24) |
    ((buffer[1] as u32) << 16) |
    ((buffer[2] as u32) << 8) |
    (buffer[3] as u32)
}

/// Put a u64 value into a mutable byte slice in network byte order
#[inline]
pub fn put_u64(buffer: &mut [u8], value: u64) {
    buffer[0] = ((value >> 56) & 0xff) as u8;
    buffer[1] = ((value >> 48) & 0xff) as u8;
    buffer[2] = ((value >> 40) & 0xff) as u8;
    buffer[3] = ((value >> 32) & 0xff) as u8;
    buffer[4] = ((value >> 24) & 0xff) as u8;
    buffer[5] = ((value >> 16) & 0xff) as u8;
    buffer[6] = ((value >> 8) & 0xff) as u8;
    buffer[7] = (value & 0xff) as u8;
}

/// Get a u64 value from a byte slice in network byte order
#[inline]
pub fn get_u64(buffer: &[u8]) -> u64 {
    ((buffer[0] as u64) << 56) |
    ((buffer[1] as u64) << 48) |
    ((buffer[2] as u64) << 40) |
    ((buffer[3] as u64) << 32) |
    ((buffer[4] as u64) << 24) |
    ((buffer[5] as u64) << 16) |
    ((buffer[6] as u64) << 8) |
    (buffer[7] as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wire_encoding() {
        let mut buf = [0u8; 8];
        
        put_u8(&mut buf, 0x12);
        assert_eq!(get_u8(&buf), 0x12);

        put_u16(&mut buf, 0x1234);
        assert_eq!(get_u16(&buf), 0x1234);

        put_u32(&mut buf, 0x12345678);
        assert_eq!(get_u32(&buf), 0x12345678);

        put_u64(&mut buf, 0x1234567890ABCDEF);
        assert_eq!(get_u64(&buf), 0x1234567890ABCDEF);
    }
}
