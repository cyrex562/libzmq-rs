#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Opcode {
    Continuation = 0,
    Text = 0x01,
    Binary = 0x02,
    Close = 0x08,
    Ping = 0x09,
    Pong = 0x0A,
}

pub const MORE_FLAG: u8 = 1;
pub const COMMAND_FLAG: u8 = 2;
