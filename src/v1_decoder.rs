use std::convert::TryFrom;

#[derive(Debug)]
pub struct Msg {
    flags: u8,
    data: Vec<u8>,
}

impl Msg {
    fn new() -> Self {
        Self {
            flags: 0,
            data: Vec::new(),
        }
    }

    fn init_size(&mut self, size: usize) -> Result<(), &'static str> {
        self.data = Vec::with_capacity(size);
        Ok(())
    }

    fn set_flags(&mut self, flags: u8) {
        self.flags = flags;
    }

    fn size(&self) -> usize {
        self.data.len()
    }

    fn data_mut(&mut self) -> &mut [u8] {
        self.data.as_mut_slice()
    }
}

pub struct V1Decoder {
    tmpbuf: [u8; 8],
    in_progress: Msg,
    max_msg_size: i64,
    state: DecoderState,
    bufsize: usize,
}

#[derive(Debug)]
enum DecoderState {
    OneByteSizeReady,
    EightByteSizeReady,
    FlagsReady,
    MessageReady,
}

impl V1Decoder {
    pub fn new(bufsize: usize, max_msg_size: i64) -> Self {
        let mut decoder = Self {
            tmpbuf: [0; 8],
            in_progress: Msg::new(),
            max_msg_size,
            state: DecoderState::OneByteSizeReady,
            bufsize,
        };
        decoder.next_step(1);
        decoder
    }

    fn next_step(&mut self, bytes: usize) {
        // In a real implementation, this would set up the next read operation
        self.state = match self.state {
            DecoderState::OneByteSizeReady => DecoderState::EightByteSizeReady,
            DecoderState::EightByteSizeReady => DecoderState::FlagsReady,
            DecoderState::FlagsReady => DecoderState::MessageReady,
            DecoderState::MessageReady => DecoderState::OneByteSizeReady,
        };
    }

    pub fn decode(&mut self, data: &[u8]) -> Result<Option<Msg>, &'static str> {
        match self.state {
            DecoderState::OneByteSizeReady => self.one_byte_size_ready(data),
            DecoderState::EightByteSizeReady => self.eight_byte_size_ready(data),
            DecoderState::FlagsReady => self.flags_ready(data),
            DecoderState::MessageReady => self.message_ready(data),
        }
    }

    fn one_byte_size_ready(&mut self, data: &[u8]) -> Result<Option<Msg>, &'static str> {
        self.tmpbuf[0] = data[0];
        
        if data[0] == u8::MAX {
            self.next_step(8);
            Ok(None)
        } else {
            if data[0] == 0 {
                return Err("Protocol error: zero-size message");
            }

            if self.max_msg_size >= 0 && i64::from(data[0] - 1) > self.max_msg_size {
                return Err("Message size too large");
            }

            self.in_progress = Msg::new();
            self.in_progress.init_size((data[0] - 1) as usize)?;
            self.next_step(1);
            Ok(None)
        }
    }

    fn eight_byte_size_ready(&mut self, data: &[u8]) -> Result<Option<Msg>, &'static str> {
        self.tmpbuf[..8].copy_from_slice(&data[..8]);
        
        let payload_length = u64::from_be_bytes(self.tmpbuf);

        if payload_length == 0 {
            return Err("Protocol error: zero-size message");
        }

        if self.max_msg_size >= 0 && (payload_length - 1) > self.max_msg_size as u64 {
            return Err("Message size too large");
        }

        let msg_size = usize::try_from(payload_length - 1)
            .map_err(|_| "Message size too large for platform")?;

        self.in_progress = Msg::new();
        self.in_progress.init_size(msg_size)?;
        self.next_step(1);
        Ok(None)
    }

    fn flags_ready(&mut self, data: &[u8]) -> Result<Option<Msg>, &'static str> {
        self.tmpbuf[0] = data[0];
        self.in_progress.set_flags(data[0] & 1); // 1 is MORE flag
        self.next_step(self.in_progress.size());
        Ok(None)
    }

    fn message_ready(&mut self, data: &[u8]) -> Result<Option<Msg>, &'static str> {
        self.in_progress.data.extend_from_slice(data);
        let msg = std::mem::replace(&mut self.in_progress, Msg::new());
        self.next_step(1);
        Ok(Some(msg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_decode() {
        let mut decoder = V1Decoder::new(64, 1024);
        
        // Test one byte size
        let result = decoder.decode(&[5]).unwrap(); // 4 byte message + 1 flag byte
        assert!(result.is_none());
        
        // Test flags
        let result = decoder.decode(&[0]).unwrap(); // no MORE flag
        assert!(result.is_none());
        
        // Test message content
        let result = decoder.decode(&[1, 2, 3, 4]).unwrap();
        assert!(result.is_some());
        let msg = result.unwrap();
        assert_eq!(msg.data, vec![1, 2, 3, 4]);
        assert_eq!(msg.flags, 0);
    }
}
