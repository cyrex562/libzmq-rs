use std::cmp;

// Trait to define the encoder interface
pub trait EncoderStep {
    fn next_step(&mut self);
}

#[derive(Debug)]
pub struct Msg {
    // Message implementation details would go here
}

impl Msg {
    pub fn close(&mut self) -> Result<(), &'static str> {
        Ok(()) // Implementation details
    }

    pub fn init(&mut self) -> Result<(), &'static str> {
        Ok(()) // Implementation details
    }
}

pub struct EncoderBase<T: EncoderStep> {
    write_pos: Vec<u8>,
    write_offset: usize,
    to_write: usize,
    next: Option<fn(&mut T)>,
    new_msg_flag: bool,
    buf_size: usize,
    buf: Vec<u8>,
    in_progress: Option<Msg>,
}

impl<T: EncoderStep> EncoderBase<T> {
    pub fn new(bufsize: usize) -> Self {
        Self {
            write_pos: Vec::new(),
            write_offset: 0,
            to_write: 0,
            next: None,
            new_msg_flag: false,
            buf_size: bufsize,
            buf: vec![0; bufsize],
            in_progress: None,
        }
    }

    pub fn encode(&mut self, data: Option<&mut Vec<u8>>) -> usize {
        let buffer = match data {
            Some(d) => d,
            None => &mut self.buf,
        };
        let buffersize = buffer.len();

        if self.in_progress.is_none() {
            return 0;
        }

        let mut pos = 0;
        while pos < buffersize {
            if self.to_write == 0 {
                if self.new_msg_flag {
                    if let Some(msg) = self.in_progress.as_mut() {
                        msg.close().expect("Failed to close message");
                        msg.init().expect("Failed to init message");
                    }
                    self.in_progress = None;
                    break;
                }
                // Call the next step implementation
                if let Some(next_fn) = self.next {
                    next_fn(unsafe { &mut *(self as *mut _ as *mut T) });
                }
            }

            // Zero-copy optimization for large messages
            if pos == 0 && data.is_none() && self.to_write >= buffersize {
                buffer.copy_from_slice(&self.write_pos[self.write_offset..self.write_offset + self.to_write]);
                pos = self.to_write;
                self.write_offset = 0;
                self.to_write = 0;
                return pos;
            }

            // Copy data to the buffer
            let to_copy = cmp::min(self.to_write, buffersize - pos);
            buffer[pos..pos + to_copy].copy_from_slice(
                &self.write_pos[self.write_offset..self.write_offset + to_copy]
            );
            pos += to_copy;
            self.write_offset += to_copy;
            self.to_write -= to_copy;
        }

        pos
    }

    pub fn load_msg(&mut self, msg: Msg) {
        assert!(self.in_progress.is_none());
        self.in_progress = Some(msg);
        if let Some(next_fn) = self.next {
            next_fn(unsafe { &mut *(self as *mut _ as *mut T) });
        }
    }

    pub fn next_step(
        &mut self,
        write_pos: Vec<u8>,
        to_write: usize,
        next: Option<fn(&mut T)>,
        new_msg_flag: bool,
    ) {
        self.write_pos = write_pos;
        self.write_offset = 0;
        self.to_write = to_write;
        self.next = next;
        self.new_msg_flag = new_msg_flag;
    }

    pub fn in_progress(&self) -> Option<&Msg> {
        self.in_progress.as_ref()
    }
}
