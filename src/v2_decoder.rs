use std::convert::TryFrom;

// Protocol constants
mod v2_protocol {
    pub const MORE_FLAG: u8 = 0x1;
    pub const COMMAND_FLAG: u8 = 0x2;
    pub const LARGE_FLAG: u8 = 0x4;
}

#[derive(Debug)]
pub struct Message {
    data: Vec<u8>,
    flags: u8,
}

impl Message {
    const MORE: u8 = 0x1;
    const COMMAND: u8 = 0x2;

    fn new() -> Self {
        Message {
            data: Vec::new(),
            flags: 0,
        }
    }

    fn init_size(&mut self, size: usize) -> Result<(), &'static str> {
        self.data = Vec::with_capacity(size);
        Ok(())
    }

    fn set_flags(&mut self, flags: u8) {
        self.flags = flags;
    }

    fn data(&self) -> &[u8] {
        &self.data
    }

    fn size(&self) -> usize {
        self.data.len()
    }
}

pub struct V2Decoder {
    tmpbuf: [u8; 8],
    msg_flags: u8,
    in_progress: Message,
    zero_copy: bool,
    max_msg_size: i64,
    buffer: Vec<u8>,
    next_step: DecoderState,
}

enum DecoderState {
    FlagsReady,
    OneByteSizeReady,
    EightByteSizeReady,
    MessageReady,
}

impl V2Decoder {
    pub fn new(bufsize: usize, maxmsgsize: i64, zero_copy: bool) -> Self {
        V2Decoder {
            tmpbuf: [0; 8],
            msg_flags: 0,
            in_progress: Message::new(),
            zero_copy,
            max_msg_size: maxmsgsize,
            buffer: Vec::with_capacity(bufsize),
            next_step: DecoderState::FlagsReady,
        }
    }

    pub fn decode(&mut self, data: &[u8]) -> Result<Option<Message>, &'static str> {
        match self.next_step {
            DecoderState::FlagsReady => self.flags_ready(data),
            DecoderState::OneByteSizeReady => self.one_byte_size_ready(data),
            DecoderState::EightByteSizeReady => self.eight_byte_size_ready(data),
            DecoderState::MessageReady => self.message_ready(data),
        }
    }

    fn flags_ready(&mut self, data: &[u8]) -> Result<Option<Message>, &'static str> {
        if data.is_empty() {
            return Ok(None);
        }

        self.msg_flags = 0;
        if data[0] & v2_protocol::MORE_FLAG != 0 {
            self.msg_flags |= Message::MORE;
        }
        if data[0] & v2_protocol::COMMAND_FLAG != 0 {
            self.msg_flags |= Message::COMMAND;
        }

        self.next_step = if data[0] & v2_protocol::LARGE_FLAG != 0 {
            DecoderState::EightByteSizeReady
        } else {
            DecoderState::OneByteSizeReady
        };

        Ok(None)
    }

    fn one_byte_size_ready(&mut self, data: &[u8]) -> Result<Option<Message>, &'static str> {
        if data.is_empty() {
            return Ok(None);
        }
        self.size_ready(data[0] as u64, data)
    }

    fn eight_byte_size_ready(&mut self, data: &[u8]) -> Result<Option<Message>, &'static str> {
        if data.len() < 8 {
            return Ok(None);
        }
        let msg_size = u64::from_be_bytes(data[0..8].try_into().unwrap());
        self.size_ready(msg_size, &data[8..])
    }

    fn size_ready(&mut self, msg_size: u64, data: &[u8]) -> Result<Option<Message>, &'static str> {
        if self.max_msg_size >= 0 && msg_size > self.max_msg_size as u64 {
            return Err("Message size too large");
        }

        let size = usize::try_from(msg_size).map_err(|_| "Message size overflow")?;
        self.in_progress.init_size(size)?;
        self.in_progress.set_flags(self.msg_flags);

        self.next_step = DecoderState::MessageReady;
        Ok(None)
    }

    fn message_ready(&mut self, data: &[u8]) -> Result<Option<Message>, &'static str> {
        self.next_step = DecoderState::FlagsReady;
        let mut completed_msg = Message::new();
        std::mem::swap(&mut self.in_progress, &mut completed_msg);
        Ok(Some(completed_msg))
    }
}
