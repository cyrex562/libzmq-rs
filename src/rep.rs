use std::error::Error;

use crate::context::Context;

// Equivalent to C++ msg_t
struct Msg {
    flags: u32,
    data: Vec<u8>,
}

impl Msg {
    const MORE: u32 = 1;

    fn new() -> Self {
        Msg {
            flags: 0,
            data: Vec::new(),
        }
    }

    fn flags(&self) -> u32 {
        self.flags
    }

    fn size(&self) -> usize {
        self.data.len()
    }
}

// Router functionality (simplified)
struct Router {
    // Router implementation details
}

impl Router {
    fn new(parent: *mut Context, tid: u32, sid: i32) -> Self {
        Router {}
    }

    fn xsend(&mut self, msg: &mut Msg) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn xrecv(&mut self, msg: &mut Msg) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn rollback(&mut self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn xhas_in(&self) -> bool {
        false
    }

    fn xhas_out(&self) -> bool {
        false
    }
}

// Main REP implementation
pub struct Rep {
    router: Router,
    sending_reply: bool,
    request_begins: bool,
}

impl Rep {
    pub fn new(parent: *mut Context, tid: u32, sid: i32) -> Self {
        Rep {
            router: Router::new(parent, tid, sid),
            sending_reply: false,
            request_begins: true,
        }
    }

    pub fn xsend(&mut self, msg: &mut Msg) -> Result<(), Box<dyn Error>> {
        if !self.sending_reply {
            return Err("EFSM".into());
        }

        let more = (msg.flags() & Msg::MORE) != 0;

        self.router.xsend(msg)?;

        if !more {
            self.sending_reply = false;
        }

        Ok(())
    }

    pub fn xrecv(&mut self, msg: &mut Msg) -> Result<(), Box<dyn Error>> {
        if self.sending_reply {
            return Err("EFSM".into());
        }

        if self.request_begins {
            loop {
                self.router.xrecv(msg)?;

                if (msg.flags() & Msg::MORE) != 0 {
                    let bottom = msg.size() == 0;
                    self.router.xsend(msg)?;

                    if bottom {
                        break;
                    }
                } else {
                    self.router.rollback()?;
                }
            }
            self.request_begins = false;
        }

        self.router.xrecv(msg)?;

        if (msg.flags() & Msg::MORE) == 0 {
            self.sending_reply = true;
            self.request_begins = true;
        }

        Ok(())
    }

    pub fn xhas_in(&self) -> bool {
        if self.sending_reply {
            return false;
        }
        self.router.xhas_in()
    }

    pub fn xhas_out(&self) -> bool {
        if !self.sending_reply {
            return false;
        }
        self.router.xhas_out()
    }
}
