use crate::message::Message;
use std::error::Error;
// #[derive(Debug)]
// pub struct Message {
//     flags: u32,
//     data: Vec<u8>,
// }
//
// impl Message {
//     pub fn new() -> Self {
//         Message {
//             flags: 0,
//             data: Vec::new(),
//         }
//     }
//
//     pub fn has_more(&self) -> bool {
//         (self.flags & 1) != 0
//     }
// }

pub trait Pipe {
    fn read(&mut self, msg: &mut Message) -> bool;
    fn check_read(&self) -> bool;
}

pub struct FairQueue {
    pipes: Vec<T>,
    active: usize,
    current: usize,
    more: bool,
}

impl<T: Pipe> FairQueue<T> {
    pub fn new() -> Self {
        FairQueue {
            pipes: Vec::new(),
            active: 0,
            current: 0,
            more: false,
        }
    }

    pub fn attach(&mut self, pipe: T) {
        self.pipes.push(pipe);
        self.pipes.swap(self.active, self.pipes.len() - 1);
        self.active += 1;
    }

    pub fn pipe_terminated(&mut self, index: usize) {
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

    pub fn recv(&mut self) -> Result<Message, Box<dyn Error>> {
        self.recv_pipe().map(|(msg, _)| msg)
    }

    pub fn recv_pipe(&mut self) -> Result<(Message, usize), Box<dyn Error>> {
        let mut msg = Message::new();

        while self.active > 0 {
            if self.pipes[self.current].read(&mut msg) {
                let current_pipe = self.current;
                self.more = msg.has_more();

                if !self.more {
                    self.current = (self.current + 1) % self.active;
                }

                return Ok((msg, current_pipe));
            }

            debug_assert!(!self.more);

            self.active -= 1;
            self.pipes.swap(self.current, self.active);
            if self.current == self.active {
                self.current = 0;
            }
        }

        Err("No messages available".into())
    }

    pub fn has_in(&mut self) -> bool {
        if self.more {
            return true;
        }

        while self.active > 0 {
            if self.pipes[self.current].check_read() {
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

#[cfg(test)]
mod tests {
    use super::*;

    struct MockPipe {
        has_message: bool,
    }

    impl Pipe for MockPipe {
        fn read(&mut self, _msg: &mut Message) -> bool {
            self.has_message
        }

        fn check_read(&self) -> bool {
            self.has_message
        }
    }

    #[test]
    fn test_fair_queue_basic() {
        let mut fq = FairQueue::new();
        assert!(!fq.has_in());

        fq.attach(MockPipe { has_message: true });
        assert!(fq.has_in());
    }
}
