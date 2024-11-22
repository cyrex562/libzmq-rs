use std::mem;

use crate::{message::Message, pipe::Pipe};

// pub struct Msg {
//     flags: u32,
// }

// impl Msg {
//     pub fn new() -> Self {
//         Self { flags: 0 }
//     }

//     pub fn flags(&self) -> u32 {
//         self.flags
//     }

//     pub fn has_more(&self) -> bool {
//         (self.flags & Self::MORE) != 0
//     }

//     const MORE: u32 = 1;
// }

// pub struct Pipe {
//     // Implementation details omitted for brevity
// }

// impl Pipe {
//     pub fn write(&mut self, _msg: &mut Msg) -> bool {
//         // Simplified implementation
//         true
//     }

//     pub fn rollback(&mut self) {
//         // Simplified implementation
//     }

//     pub fn flush(&mut self) {
//         // Simplified implementation
//     }

//     pub fn check_write(&self) -> bool {
//         // Simplified implementation
//         true
//     }
// }

pub struct LoadBalancer {
    pipes: Vec<Pipe>,
    active: usize,
    current: usize,
    more: bool,
    dropping: bool,
}

impl LoadBalancer {
    pub fn new() -> Self {
        Self {
            pipes: Vec::with_capacity(2),
            active: 0,
            current: 0,
            more: false,
            dropping: false,
        }
    }

    pub fn attach(&mut self, pipe: Pipe) {
        self.pipes.push(pipe);
        self.activated(self.pipes.len() - 1);
    }

    pub fn pipe_terminated(&mut self, index: usize) {
        if index == self.current && self.more {
            self.dropping = true;
        }

        if index < self.active {
            self.active -= 1;
            self.pipes.swap(index, self.active);
            if self.current == self.active {
                self.current = 0;
            }
        }
        self.pipes.remove(index);
    }

    pub fn activated(&mut self, index: usize) {
        self.pipes.swap(index, self.active);
        self.active += 1;
    }

    pub fn send(&mut self, msg: &mut Message) -> Result<(), i32> {
        self.sendpipe(msg, None)
    }

    pub fn sendpipe(&mut self, msg: &mut Message, pipe: Option<&mut Pipe>) -> Result<(), i32> {
        if self.dropping {
            self.more = msg.has_more();
            self.dropping = self.more;
            return Ok(());
        }

        while self.active > 0 {
            if self.pipes[self.current].write(msg) {
                if let Some(p) = pipe {
                    *p = mem::replace(&mut self.pipes[self.current], Pipe::new());
                }
                break;
            }

            if self.more {
                self.pipes[self.current].rollback();
                self.dropping = msg.has_more();
                self.more = false;
                return Err(-2); // EAGAIN equivalent
            }

            self.active -= 1;
            if self.current < self.active {
                self.pipes.swap(self.current, self.active);
            } else {
                self.current = 0;
            }
        }

        if self.active == 0 {
            return Err(-1); // EAGAIN equivalent
        }

        self.more = msg.has_more();
        if !self.more {
            self.pipes[self.current].flush();

            self.current += 1;
            if self.current >= self.active {
                self.current = 0;
            }
        }

        Ok(())
    }

    pub fn has_out(&mut self) -> bool {
        if self.more {
            return true;
        }

        while self.active > 0 {
            if self.pipes[self.current].check_write() {
                return true;
            }

            self.active -= 1;
            self.pipes.swap(self.current, self.active);
            if self.current == self.active {
                self.current = 0;
            }
        }

        false
    }
}
