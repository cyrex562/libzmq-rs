use std::convert::TryFrom;
use std::mem;

// WebSocket protocol constants
const WS_FINAL_FRAME: u8 = 0x80;
const WS_MASK_BIT: u8 = 0x80;
const WS_LENGTH_MASK: u8 = 0x7F;
const WS_MORE_FLAG: u8 = 0x1;
const WS_COMMAND_FLAG: u8 = 0x2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpCode {
    Binary = 0x2,
    Close = 0x8,
    Ping = 0x9,
    Pong = 0xA,
}

impl TryFrom<u8> for OpCode {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value & 0xF {
            0x2 => Ok(OpCode::Binary),
            0x8 => Ok(OpCode::Close),
            0x9 => Ok(OpCode::Ping),
            0xA => Ok(OpCode::Pong),
            _ => Err("Invalid OpCode"),
        }
    }
}

#[derive(Debug)]
enum DecoderState {
    OpCode,
    SizeFirstByte,
    ShortSize,
    LongSize,
    Mask,
    Flags,
    Message,
}

#[derive(Debug)]
pub struct Message {
    data: Vec<u8>,
    flags: u8,
}

pub struct WsDecoder {
    state: DecoderState,
    tmpbuf: [u8; 8],
    msg_flags: u8,
    zero_copy: bool,
    max_msg_size: i64,
    must_mask: bool,
    size: u64,
    opcode: OpCode,
    mask: [u8; 4],
    in_progress: Option<Message>,
}

impl WsDecoder {
    pub fn new(max_msg_size: i64, zero_copy: bool, must_mask: bool) -> Self {
        Self {
            state: DecoderState::OpCode,
            tmpbuf: [0; 8],
            msg_flags: 0,
            zero_copy,
            max_msg_size,
            must_mask,
            size: 0,
            opcode: OpCode::Binary,
            mask: [0; 4],
            in_progress: None,
        }
    }

    pub fn decode(&mut self, data: &[u8]) -> Result<Option<Message>, &'static str> {
        let mut read_pos = 0;

        while read_pos < data.len() {
            match self.state {
                DecoderState::OpCode => {
                    self.tmpbuf[0] = data[read_pos];
                    read_pos += 1;

                    if (self.tmpbuf[0] & WS_FINAL_FRAME) == 0 {
                        return Err("Non-final messages not supported");
                    }

                    self.opcode = OpCode::try_from(self.tmpbuf[0])?;
                    self.msg_flags = match self.opcode {
                        OpCode::Binary => 0,
                        OpCode::Close => 0x3, // command | close
                        OpCode::Ping => 0x5,  // ping | command
                        OpCode::Pong => 0x6,  // pong | command
                    };

                    self.state = DecoderState::SizeFirstByte;
                }

                DecoderState::SizeFirstByte => {
                    self.tmpbuf[0] = data[read_pos];
                    read_pos += 1;

                    let is_masked = (self.tmpbuf[0] & WS_MASK_BIT) != 0;
                    if is_masked != self.must_mask {
                        return Err("Invalid mask flag");
                    }

                    self.size = (self.tmpbuf[0] & WS_LENGTH_MASK) as u64;

                    match self.size {
                        0..=125 => {
                            if self.must_mask {
                                self.state = DecoderState::Mask;
                            } else if self.opcode == OpCode::Binary {
                                if self.size == 0 {
                                    return Err("Zero size message");
                                }
                                self.state = DecoderState::Flags;
                            } else {
                                self.state = DecoderState::Message;
                            }
                        }
                        126 => self.state = DecoderState::ShortSize,
                        127 => self.state = DecoderState::LongSize,
                        _ => unreachable!(),
                    }
                }

                DecoderState::ShortSize => {
                    if read_pos + 2 > data.len() {
                        return Ok(None);
                    }
                    self.tmpbuf[..2].copy_from_slice(&data[read_pos..read_pos + 2]);
                    read_pos += 2;
                    self.size = u16::from_be_bytes(self.tmpbuf[..2].try_into().unwrap()) as u64;

                    if self.must_mask {
                        self.state = DecoderState::Mask;
                    } else if self.opcode == OpCode::Binary {
                        if self.size == 0 {
                            return Err("Zero size message");
                        }
                        self.state = DecoderState::Flags;
                    } else {
                        self.state = DecoderState::Message;
                    }
                }

                DecoderState::LongSize => {
                    if read_pos + 8 > data.len() {
                        return Ok(None);
                    }
                    self.tmpbuf.copy_from_slice(&data[read_pos..read_pos + 8]);
                    read_pos += 8;
                    self.size = u64::from_be_bytes(self.tmpbuf);

                    if self.must_mask {
                        self.state = DecoderState::Mask;
                    } else if self.opcode == OpCode::Binary {
                        if self.size == 0 {
                            return Err("Zero size message");
                        }
                        self.state = DecoderState::Flags;
                    } else {
                        self.state = DecoderState::Message;
                    }
                }

                DecoderState::Mask => {
                    if read_pos + 4 > data.len() {
                        return Ok(None);
                    }
                    self.mask.copy_from_slice(&data[read_pos..read_pos + 4]);
                    read_pos += 4;

                    if self.opcode == OpCode::Binary {
                        if self.size == 0 {
                            return Err("Zero size message");
                        }
                        self.state = DecoderState::Flags;
                    } else {
                        self.state = DecoderState::Message;
                    }
                }

                DecoderState::Flags => {
                    if read_pos >= data.len() {
                        return Ok(None);
                    }

                    let flags = if self.must_mask {
                        data[read_pos] ^ self.mask[0]
                    } else {
                        data[read_pos]
                    };
                    read_pos += 1;

                    if flags & WS_MORE_FLAG != 0 {
                        self.msg_flags |= WS_MORE_FLAG;
                    }
                    if flags & WS_COMMAND_FLAG != 0 {
                        self.msg_flags |= WS_COMMAND_FLAG;
                    }

                    self.size -= 1;
                    self.state = DecoderState::Message;
                }

                DecoderState::Message => {
                    if self.max_msg_size >= 0 && self.size > self.max_msg_size as u64 {
                        return Err("Message too large");
                    }

                    if read_pos + self.size as usize > data.len() {
                        return Ok(None);
                    }

                    let mut msg_data = vec![0; self.size as usize];
                    msg_data.copy_from_slice(&data[read_pos..read_pos + self.size as usize]);

                    if self.must_mask {
                        let mask_start = if self.opcode == OpCode::Binary { 1 } else { 0 };
                        for (i, byte) in msg_data.iter_mut().enumerate() {
                            *byte ^= self.mask[(mask_start + i) % 4];
                        }
                    }

                    let message = Message {
                        data: msg_data,
                        flags: self.msg_flags,
                    };

                    self.state = DecoderState::OpCode;
                    return Ok(Some(message));
                }
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_decoding() {
        let mut decoder = WsDecoder::new(1024, false, true);
        
        // Create a simple masked binary frame
        let frame = vec![
            0x82, // Final frame, binary opcode
            0x84, // Masked, payload length 4
            0x11, 0x22, 0x33, 0x44, // Mask key
            0x55, 0x66, 0x77, 0x88  // Masked payload
        ];

        let result = decoder.decode(&frame).unwrap().unwrap();
        assert_eq!(result.flags, 0);
        assert_eq!(result.data.len(), 4);
    }
}
