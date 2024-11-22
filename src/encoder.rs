use std::cmp;

pub trait EncoderStep {
    fn execute(&mut self, encoder: &mut Encoder<Self>)
    where
        Self: Sized;
}

pub trait IEncoder {
    fn encode(&mut self, data: &mut Option<&mut [u8]>, size: usize) -> usize;
    fn load_msg(&mut self, msg: Box<Message>);
}

pub struct Message {
    // Message implementation details...
}

impl Message {
    pub fn close(&mut self) -> i32 {
        0
    }
    pub fn init(&mut self) -> i32 {
        0
    }
}

pub struct Encoder<T: EncoderStep> {
    write_pos: Option<*mut u8>,
    to_write: usize,
    next_step: Option<T>,
    new_msg_flag: bool,
    buf_size: usize,
    buf: Vec<u8>,
    in_progress: Option<Box<Message>>,
}

impl<T: EncoderStep> Encoder<T> {
    pub fn new(bufsize: usize) -> Self {
        Encoder {
            write_pos: None,
            to_write: 0,
            next_step: None,
            new_msg_flag: false,
            buf_size: bufsize,
            buf: vec![0; bufsize],
            in_progress: None,
        }
    }

    pub fn next_step(&mut self, write_pos: *mut u8, to_write: usize, next: T, new_msg_flag: bool) {
        self.write_pos = Some(write_pos);
        self.to_write = to_write;
        self.next_step = Some(next);
        self.new_msg_flag = new_msg_flag;
    }

    pub fn in_progress(&self) -> Option<&Box<Message>> {
        self.in_progress.as_ref()
    }
}

impl<T: EncoderStep> IEncoder for Encoder<T> {
    fn encode(&mut self, data: &mut Option<&mut [u8]>, size: usize) -> usize {
        if self.in_progress.is_none() {
            return 0;
        }

        let buffer = match data {
            None => self.buf.as_mut_slice(),
            Some(b) => *b,
        };
        let buffersize = if data.is_none() { self.buf_size } else { size };

        let mut pos = 0;
        while pos < buffersize {
            if self.to_write == 0 {
                if self.new_msg_flag {
                    if let Some(msg) = &mut self.in_progress {
                        msg.close();
                        msg.init();
                    }
                    self.in_progress = None;
                    break;
                }
                if let Some(step) = &mut self.next_step {
                    step.execute(self);
                }
            }

            if pos == 0 && data.is_none() && self.to_write >= buffersize {
                if let Some(write_pos) = self.write_pos {
                    *data =
                        Some(unsafe { std::slice::from_raw_parts_mut(write_pos, self.to_write) });
                    pos = self.to_write;
                    self.write_pos = None;
                    self.to_write = 0;
                    return pos;
                }
            }

            let to_copy = cmp::min(self.to_write, buffersize - pos);
            if let Some(write_pos) = self.write_pos {
                unsafe {
                    std::ptr::copy_nonoverlapping(write_pos, buffer[pos..].as_mut_ptr(), to_copy);
                }
            }
            pos += to_copy;
            self.write_pos = self.write_pos.map(|p| unsafe { p.add(to_copy) });
            self.to_write -= to_copy;
        }

        *data = Some(buffer);
        pos
    }

    fn load_msg(&mut self, msg: Box<Message>) {
        assert!(self.in_progress.is_none());
        self.in_progress = Some(msg);
        if let Some(step) = &mut self.next_step {
            step.execute(self);
        }
    }
}
