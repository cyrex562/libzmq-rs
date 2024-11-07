use std::cmp::min;

// Constants from v2_protocol
const MORE_FLAG: u8 = 0x1;
const LARGE_FLAG: u8 = 0x2;
const COMMAND_FLAG: u8 = 0x4;

pub struct Message {
    data: Vec<u8>,
    flags: u8,
    is_subscribe: bool,
    is_cancel: bool,
}

impl Message {
    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn has_more(&self) -> bool {
        (self.flags & MORE_FLAG) == MORE_FLAG
    }

    pub fn is_command(&self) -> bool {
        (self.flags & COMMAND_FLAG) == COMMAND_FLAG
    }
}

pub struct V2Encoder {
    tmp_buf: [u8; 10],
    buffer_size: usize,
    in_progress: Option<Message>,
}

impl V2Encoder {
    pub fn new(buffer_size: usize) -> Self {
        V2Encoder {
            tmp_buf: [0; 10],
            buffer_size,
            in_progress: None,
        }
    }

    pub fn encode_message(&mut self, msg: Message) -> Vec<u8> {
        self.in_progress = Some(msg);
        let mut result = Vec::new();
        
        // Encode header
        let size = self.in_progress.as_ref().unwrap().size();
        let mut header_size = 2; // flags byte + size byte
        
        // Encode flags
        let mut protocol_flags: u8 = 0;
        let msg = self.in_progress.as_ref().unwrap();
        
        if msg.has_more() {
            protocol_flags |= MORE_FLAG;
        }
        if size > u8::MAX as usize {
            protocol_flags |= LARGE_FLAG;
        }
        if msg.is_command() {
            protocol_flags |= COMMAND_FLAG;
        }
        
        self.tmp_buf[0] = protocol_flags;

        // Encode message length
        if size > u8::MAX as usize {
            self.tmp_buf[1..9].copy_from_slice(&(size as u64).to_be_bytes());
            header_size = 9;
        } else {
            self.tmp_buf[1] = size as u8;
        }

        // Encode subscribe/cancel byte
        if msg.is_subscribe {
            self.tmp_buf[header_size] = 1;
            header_size += 1;
        } else if msg.is_cancel {
            self.tmp_buf[header_size] = 0;
            header_size += 1;
        }

        // Add header to result
        result.extend_from_slice(&self.tmp_buf[0..header_size]);
        
        // Add message body
        result.extend_from_slice(&msg.data);

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_encoding() {
        let mut encoder = V2Encoder::new(1024);
        let msg = Message {
            data: vec![1, 2, 3],
            flags: 0,
            is_subscribe: false,
            is_cancel: false,
        };
        
        let result = encoder.encode_message(msg);
        assert_eq!(result.len(), 5); // 2 bytes header + 3 bytes data
        assert_eq!(result[0], 0); // no flags
        assert_eq!(result[1], 3); // size
        assert_eq!(&result[2..], &[1, 2, 3]); // data
    }
}
