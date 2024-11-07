use std::convert::TryFrom;

// WebSocket protocol constants
pub mod ws_protocol {
    pub const OPCODE_PING: u8 = 0x9;
    pub const OPCODE_PONG: u8 = 0xA;
    pub const OPCODE_CLOSE: u8 = 0x8;
    pub const MORE_FLAG: u8 = 0x1;
    pub const COMMAND_FLAG: u8 = 0x2;
}

#[derive(Debug)]
pub struct WsEncoder {
    buf_size: usize,
    must_mask: bool,
    tmp_buf: [u8; 16],
    mask: [u8; 4],
    masked_msg: Option<Vec<u8>>,
    is_binary: bool,
}

impl WsEncoder {
    pub fn new(buf_size: usize, must_mask: bool) -> Self {
        WsEncoder {
            buf_size,
            must_mask,
            tmp_buf: [0; 16],
            mask: [0; 4],
            masked_msg: None,
            is_binary: false,
        }
    }

    pub fn encode_message(&mut self, msg: &Message) -> Vec<u8> {
        let mut offset = 0;
        self.is_binary = false;

        // Set opcode
        self.tmp_buf[offset] = if msg.is_ping() {
            0x80 | ws_protocol::OPCODE_PING
        } else if msg.is_pong() {
            0x80 | ws_protocol::OPCODE_PONG
        } else if msg.is_close_cmd() {
            0x80 | ws_protocol::OPCODE_CLOSE
        } else {
            self.is_binary = true;
            0x82 // Final | binary
        };
        offset += 1;

        // Set mask bit and payload length
        self.tmp_buf[offset] = if self.must_mask { 0x80 } else { 0x00 };
        
        let mut size = msg.size();
        if self.is_binary {
            size += 1;
        }
        if msg.is_subscribe() || msg.is_cancel() {
            size += 1;
        }

        if size <= 125 {
            self.tmp_buf[offset] |= (size & 127) as u8;
            offset += 1;
        } else if size <= 0xFFFF {
            self.tmp_buf[offset] |= 126;
            offset += 1;
            self.tmp_buf[offset..offset + 2].copy_from_slice(&(size as u16).to_be_bytes());
            offset += 2;
        } else {
            self.tmp_buf[offset] |= 127;
            offset += 1;
            self.tmp_buf[offset..offset + 8].copy_from_slice(&(size as u64).to_be_bytes());
            offset += 8;
        }

        // Generate and set mask if required
        if self.must_mask {
            let random = rand::random::<u32>();
            self.mask.copy_from_slice(&random.to_be_bytes());
            self.tmp_buf[offset..offset + 4].copy_from_slice(&self.mask);
            offset += 4;
        }

        // Create result vector and add header
        let mut result = Vec::with_capacity(offset + size);
        result.extend_from_slice(&self.tmp_buf[..offset]);

        // Add protocol flags if binary
        let mut mask_index = 0;
        if self.is_binary {
            let protocol_flags = {
                let mut flags = 0u8;
                if msg.has_more() {
                    flags |= ws_protocol::MORE_FLAG;
                }
                if msg.is_command() {
                    flags |= ws_protocol::COMMAND_FLAG;
                }
                flags
            };

            if self.must_mask {
                result.push(protocol_flags ^ self.mask[mask_index]);
                mask_index += 1;
            } else {
                result.push(protocol_flags);
            }
        }

        // Add subscribe/cancel byte if needed
        if msg.is_subscribe() {
            result.push(if self.must_mask { 1 ^ self.mask[mask_index] } else { 1 });
            mask_index += 1;
        } else if msg.is_cancel() {
            result.push(if self.must_mask { 0 ^ self.mask[mask_index] } else { 0 });
            mask_index += 1;
        }

        // Add payload data
        if self.must_mask {
            let data = msg.data();
            for (i, &byte) in data.iter().enumerate() {
                result.push(byte ^ self.mask[(mask_index + i) % 4]);
            }
        } else {
            result.extend_from_slice(msg.data());
        }

        result
    }
}

// Message type stub - you'll need to implement this based on your needs
#[derive(Debug)]
pub struct Message {
    data: Vec<u8>,
    flags: u8,
}

impl Message {
    fn size(&self) -> usize {
        self.data.len()
    }

    fn data(&self) -> &[u8] {
        &self.data
    }

    fn is_ping(&self) -> bool {
        // Implementation depends on your message format
        false
    }

    fn is_pong(&self) -> bool {
        // Implementation depends on your message format
        false
    }

    fn is_close_cmd(&self) -> bool {
        // Implementation depends on your message format
        false
    }

    fn is_subscribe(&self) -> bool {
        // Implementation depends on your message format
        false
    }

    fn is_cancel(&self) -> bool {
        // Implementation depends on your message format
        false
    }

    fn has_more(&self) -> bool {
        self.flags & ws_protocol::MORE_FLAG != 0
    }

    fn is_command(&self) -> bool {
        self.flags & ws_protocol::COMMAND_FLAG != 0
    }
}
