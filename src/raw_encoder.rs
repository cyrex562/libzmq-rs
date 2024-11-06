#![allow(dead_code)]

// Equivalent to encoder_base_t from C++
pub trait EncoderBase {
    fn next_step(&mut self, data: Option<&[u8]>, size: usize, ready: bool);
    fn in_progress(&mut self) -> &mut Vec<u8>;
}

pub struct RawEncoder {
    buffer: Vec<u8>,
    buffer_size: usize,
}

impl RawEncoder {
    pub fn new(bufsize: usize) -> Self {
        let mut encoder = RawEncoder {
            buffer: Vec::with_capacity(bufsize),
            buffer_size: bufsize,
        };
        // Initialize with empty data and ready state
        encoder.next_step(None, 0, true);
        encoder
    }

    fn raw_message_ready(&mut self) {
        let data = self.in_progress();
        self.next_step(Some(data), data.len(), true);
    }
}

impl EncoderBase for RawEncoder {
    fn next_step(&mut self, data: Option<&[u8]>, size: usize, ready: bool) {
        if let Some(data) = data {
            self.buffer.clear();
            self.buffer.extend_from_slice(data);
        }
    }

    fn in_progress(&mut self) -> &mut Vec<u8> {
        &mut self.buffer
    }
}
