#![allow(dead_code)]

/// SHA1 context structure
pub struct Sha1Context {
    h: [u32; 5],
    len: u64,
    block: [u8; 64],
    block_len: u8,
}

const K: [u32; 4] = [0x5a827999, 0x6ed9eba1, 0x8f1bbcdc, 0xca62c1d6];

impl Sha1Context {
    pub fn new() -> Self {
        let mut ctx = Sha1Context {
            h: [0; 5],
            len: 0,
            block: [0; 64],
            block_len: 0,
        };
        ctx.init();
        ctx
    }

    fn init(&mut self) {
        self.h[0] = 0x67452301;
        self.h[1] = 0xefcdab89;
        self.h[2] = 0x98badcfe;
        self.h[3] = 0x10325476;
        self.h[4] = 0xc3d2e1f0;
        self.len = 0;
        self.block_len = 0;
    }

    fn step(&mut self) {
        let mut w = [0u32; 80];

        // Convert block to words
        for i in 0..16 {
            let off = i * 4;
            w[i] = ((self.block[off] as u32) << 24)
                | ((self.block[off + 1] as u32) << 16)
                | ((self.block[off + 2] as u32) << 8)
                | (self.block[off + 3] as u32);
        }

        // Extend words
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }

        let mut a = self.h[0];
        let mut b = self.h[1];
        let mut c = self.h[2];
        let mut d = self.h[3];
        let mut e = self.h[4];

        // Main loop
        for t in 0..80 {
            let temp = a.rotate_left(5)
                + match t {
                    0..=19 => (b & c) | (!b & d) + K[0],
                    20..=39 => (b ^ c ^ d) + K[1],
                    40..=59 => (b & c) | (b & d) | (c & d) + K[2],
                    60..=79 => (b ^ c ^ d) + K[3],
                    _ => unreachable!(),
                }
                + e
                + w[t];

            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        self.h[0] = self.h[0].wrapping_add(a);
        self.h[1] = self.h[1].wrapping_add(b);
        self.h[2] = self.h[2].wrapping_add(c);
        self.h[3] = self.h[3].wrapping_add(d);
        self.h[4] = self.h[4].wrapping_add(e);
    }

    pub fn update(&mut self, input: &[u8]) {
        let mut input_len = input.len();
        let mut input_idx = 0;

        self.len += input_len as u64;

        if self.block_len > 0 {
            let available = 64 - self.block_len as usize;
            let to_copy = input_len.min(available);
            self.block[self.block_len as usize..self.block_len as usize + to_copy]
                .copy_from_slice(&input[..to_copy]);
            self.block_len += to_copy as u8;
            input_len -= to_copy;
            input_idx += to_copy;

            if self.block_len == 64 {
                self.step();
                self.block_len = 0;
            }
        }

        while input_len >= 64 {
            self.block.copy_from_slice(&input[input_idx..input_idx + 64]);
            self.step();
            input_len -= 64;
            input_idx += 64;
        }

        if input_len > 0 {
            self.block[..input_len].copy_from_slice(&input[input_idx..]);
            self.block_len = input_len as u8;
        }
    }

    pub fn finalize(&mut self) -> [u8; 20] {
        let mut result = [0u8; 20];
        let bit_len = self.len * 8;

        // Padding
        self.block[self.block_len as usize] = 0x80;
        self.block_len += 1;

        if self.block_len > 56 {
            self.block[self.block_len as usize..].fill(0);
            self.step();
            self.block_len = 0;
        }

        self.block[self.block_len as usize..56].fill(0);
        
        // Append length
        self.block[56..64].copy_from_slice(&bit_len.to_be_bytes());
        self.step();

        // Convert hash to bytes
        for (i, word) in self.h.iter().enumerate() {
            let off = i * 4;
            result[off..off + 4].copy_from_slice(&word.to_be_bytes());
        }

        result
    }
}

// OpenSSL compatibility interface
pub const SHA_DIGEST_LENGTH: usize = 20;

pub struct SHA_CTX(Sha1Context);

impl SHA_CTX {
    pub fn new() -> Self {
        SHA_CTX(Sha1Context::new())
    }

    pub fn init(&mut self) {
        self.0.init();
    }

    pub fn update(&mut self, data: &[u8]) {
        self.0.update(data);
    }

    pub fn finish(&mut self, output: &mut [u8; SHA_DIGEST_LENGTH]) {
        output.copy_from_slice(&self.0.finalize());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_string() {
        let mut sha1 = Sha1Context::new();
        let result = sha1.finalize();
        assert_eq!(
            hex::encode(result),
            "da39a3ee5e6b4b0d3255bfef95601890afd80709"
        );
    }

    #[test]
    fn test_hello_world() {
        let mut sha1 = Sha1Context::new();
        sha1.update(b"Hello, world!");
        let result = sha1.finalize();
        assert_eq!(
            hex::encode(result),
            "0a0a9f2a6772942557ab5355d76af442f8f65e01"
        );
    }
}
