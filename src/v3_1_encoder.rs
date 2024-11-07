use std::cmp::min;

const SUB_CMD_NAME_SIZE: usize = 4;  // Assumed size, adjust as needed
const CANCEL_CMD_NAME_SIZE: usize = 6;  // Assumed size, adjust as needed
const SUB_CMD_NAME: &[u8] = b"SUB\0";  // Example command name
const CANCEL_CMD_NAME: &[u8] = b"CANCEL";

#[derive(Debug)]
pub struct Msg {
    flags: u8,
    data: Vec<u8>,
}

impl Msg {
    const MORE: u8 = 0x1;
    const COMMAND: u8 = 0x2;

    pub fn new() -> Self {
        Msg {
            flags: 0,
            data: Vec::new(),
        }
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn flags(&self) -> u8 {
        self.flags
    }

    pub fn is_subscribe(&self) -> bool {
        (self.flags & Self::COMMAND) != 0 && self.data.starts_with(SUB_CMD_NAME)
    }

    pub fn is_cancel(&self) -> bool {
        (self.flags & Self::COMMAND) != 0 && self.data.starts_with(CANCEL_CMD_NAME)
    }
}

struct V31Encoder {
    tmp_buf: [u8; 9 + SUB_CMD_NAME_SIZE],
    in_progress: Option<Msg>,
    buffer: Vec<u8>,
    buffer_size: usize,
    bytes_written: usize,
}

impl V31Encoder {
    pub fn new(bufsize: usize) -> Self {
        let mut encoder = V31Encoder {
            tmp_buf: [0; 9 + SUB_CMD_NAME_SIZE],
            in_progress: None,
            buffer: Vec::with_capacity(bufsize),
            buffer_size: bufsize,
            bytes_written: 0,
        };
        encoder.next_step();
        encoder
    }

    fn next_step(&mut self) {
        if let Some(msg) = &self.in_progress {
            self.message_ready();
        }
    }

    fn message_ready(&mut self) {
        if let Some(msg) = &self.in_progress {
            // Encode flags
            let mut size = msg.size();
            let mut header_size = 2; // flags byte + size byte
            let mut protocol_flags = 0u8;

            if (msg.flags() & Msg::MORE) != 0 {
                protocol_flags |= 0x1; // more_flag
            }

            if (msg.flags() & Msg::COMMAND) != 0 || msg.is_subscribe() || msg.is_cancel() {
                protocol_flags |= 0x4; // command_flag
                if msg.is_subscribe() {
                    size += SUB_CMD_NAME_SIZE;
                } else if msg.is_cancel() {
                    size += CANCEL_CMD_NAME_SIZE;
                }
            }

            // Large message flag
            if size > u8::MAX as usize {
                protocol_flags |= 0x2; // large_flag
            }

            self.tmp_buf[0] = protocol_flags;

            // Encode message length
            if size > u8::MAX as usize {
                self.encode_u64(size as u64, &mut self.tmp_buf[1..9]);
                header_size = 9;
            } else {
                self.tmp_buf[1] = size as u8;
            }

            // Encode sub/cancel command string
            if msg.is_subscribe() {
                self.tmp_buf[header_size..header_size + SUB_CMD_NAME_SIZE]
                    .copy_from_slice(SUB_CMD_NAME);
                header_size += SUB_CMD_NAME_SIZE;
            } else if msg.is_cancel() {
                self.tmp_buf[header_size..header_size + CANCEL_CMD_NAME_SIZE]
                    .copy_from_slice(CANCEL_CMD_NAME);
                header_size += CANCEL_CMD_NAME_SIZE;
            }

            self.size_ready(header_size);
        }
    }

    fn size_ready(&mut self, header_size: usize) {
        if let Some(msg) = &self.in_progress {
            // Write header
            self.buffer.extend_from_slice(&self.tmp_buf[..header_size]);
            // Write message body
            self.buffer.extend_from_slice(&msg.data);
            self.bytes_written = header_size + msg.size();
        }
    }

    fn encode_u64(&mut self, value: u64, buf: &mut [u8]) {
        buf.copy_from_slice(&value.to_be_bytes());
    }
}
