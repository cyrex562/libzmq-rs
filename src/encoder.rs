use std::cmp::min;

// Constants from v2_protocol
const MORE_FLAG: u8 = 0x1;
const LARGE_FLAG: u8 = 0x2;
const COMMAND_FLAG: u8 = 0x4;

// Message type equivalent
struct Msg {
    data: Vec<u8>,
    flags: u32,
}

impl Msg {
    fn size(&self) -> usize {
        self.data.len()
    }

    fn flags(&self) -> u32 {
        self.flags
    }

    fn is_subscribe(&self) -> bool {
        // Implementation depends on how subscribe messages are marked
        false
    }

    fn is_cancel(&self) -> bool {
        // Implementation depends on how cancel messages are marked
        false
    }

    fn data(&self) -> &[u8] {
        &self.data
    }
}

pub struct V2Encoder {
    tmp_buf: [u8; 10],  // flags byte + size byte (or 8 bytes) + sub/cancel byte
    buffer: Vec<u8>,
    buf_size: usize,
    in_progress: Option<Msg>,
    next_step: fn(&mut V2Encoder),
    write_pos: usize,
}

impl V2Encoder {
    pub fn new(bufsize: usize) -> Self {
        let mut encoder = V2Encoder {
            tmp_buf: [0; 10],
            buffer: vec![0; bufsize],
            buf_size: bufsize,
            in_progress: None,
            next_step: Self::message_ready,
            write_pos: 0,
        };
        encoder.next_step = Self::message_ready;
        encoder
    }

    fn next_step(&mut self, data: Option<&[u8]>, size: usize, next: fn(&mut V2Encoder), copy_msg: bool) {
        if let Some(data) = data {
            let to_copy = min(size, self.buf_size - self.write_pos);
            self.buffer[self.write_pos..self.write_pos + to_copy]
                .copy_from_slice(&data[..to_copy]);
            self.write_pos += to_copy;
        }
        self.next_step = next;
        if copy_msg {
            self.in_progress = None;
        }
    }

    fn message_ready(&mut self) {
        if let Some(msg) = &self.in_progress {
            let mut size = msg.size();
            let mut header_size = 2; // flags byte + size byte
            
            // Encode flags
            let mut protocol_flags = 0u8;
            if msg.flags() & 1 != 0 { // MORE flag
                protocol_flags |= MORE_FLAG;
            }
            if msg.size() > u8::MAX as usize {
                protocol_flags |= LARGE_FLAG;
            }
            if msg.flags() & 4 != 0 { // COMMAND flag
                protocol_flags |= COMMAND_FLAG;
            }
            
            self.tmp_buf[0] = protocol_flags;

            if msg.is_subscribe() || msg.is_cancel() {
                size += 1;
            }

            // Encode message length
            if size > u8::MAX as usize {
                self.tmp_buf[1..9].copy_from_slice(&(size as u64).to_be_bytes());
                header_size = 9;
            } else {
                self.tmp_buf[1] = size as u8;
            }

            // Encode subscribe/cancel byte
            if msg.is_subscribe() {
                self.tmp_buf[header_size] = 1;
                header_size += 1;
            } else if msg.is_cancel() {
                self.tmp_buf[header_size] = 0;
                header_size += 1;
            }

            self.next_step(Some(&self.tmp_buf[..header_size]), header_size, Self::size_ready, false);
        }
    }

    fn size_ready(&mut self) {
        if let Some(msg) = &self.in_progress {
            self.next_step(Some(msg.data()), msg.size(), Self::message_ready, true);
        }
    }
}
