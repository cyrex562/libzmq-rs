use std::cmp::min;

// Represents a message with flags and data
pub struct Message {
    flags: u8,
    data: Vec<u8>,
    is_subscribe: bool,
    is_cancel: bool,
}

impl Message {
    const MORE: u8 = 1;

    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn flags(&self) -> u8 {
        self.flags
    }

    pub fn is_subscribe(&self) -> bool {
        self.is_subscribe
    }

    pub fn is_cancel(&self) -> bool {
        self.is_cancel
    }
}

pub struct V1Encoder {
    tmpbuf: [u8; 11],
    bufsize: usize,
    in_progress: Option<Message>,
    next_step: EncoderState,
}

enum EncoderState {
    MessageReady,
    SizeReady,
}

impl V1Encoder {
    pub fn new(bufsize: usize) -> Self {
        V1Encoder {
            tmpbuf: [0; 11],
            bufsize,
            in_progress: None,
            next_step: EncoderState::MessageReady,
        }
    }

    pub fn encode(&mut self, msg: Message) -> Vec<u8> {
        self.in_progress = Some(msg);
        let mut output = Vec::new();
        
        match self.next_step {
            EncoderState::MessageReady => {
                let msg = self.in_progress.as_ref().unwrap();
                let mut header_size = 2; // flags byte + size byte
                let mut size = msg.size();

                // Account for the 'flags' byte
                size += 1;

                // Account for the subscribe/cancel byte
                if msg.is_subscribe() || msg.is_cancel() {
                    size += 1;
                }

                // For messages less than 255 bytes long, write one byte of message size
                if size < u8::MAX as usize {
                    self.tmpbuf[0] = size as u8;
                    self.tmpbuf[1] = msg.flags() & Message::MORE;
                } else {
                    self.tmpbuf[0] = u8::MAX;
                    Self::put_uint64(&mut self.tmpbuf[1..], size as u64);
                    self.tmpbuf[9] = msg.flags() & Message::MORE;
                    header_size = 10;
                }

                // Encode the subscribe/cancel byte
                if msg.is_subscribe() {
                    self.tmpbuf[header_size] = 1;
                    header_size += 1;
                } else if msg.is_cancel() {
                    self.tmpbuf[header_size] = 0;
                    header_size += 1;
                }

                output.extend_from_slice(&self.tmpbuf[..header_size]);
                self.next_step = EncoderState::SizeReady;
            }
            EncoderState::SizeReady => {
                if let Some(msg) = &self.in_progress {
                    output.extend_from_slice(&msg.data);
                }
                self.next_step = EncoderState::MessageReady;
            }
        }

        output
    }

    fn put_uint64(buf: &mut [u8], value: u64) {
        buf[0..8].copy_from_slice(&value.to_be_bytes());
    }
}
