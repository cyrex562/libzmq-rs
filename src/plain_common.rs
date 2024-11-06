pub mod plain {
    pub const HELLO_PREFIX: &[u8] = b"\x05HELLO";
    pub const HELLO_PREFIX_LEN: usize = HELLO_PREFIX.len();

    pub const WELCOME_PREFIX: &[u8] = b"\x07WELCOME";
    pub const WELCOME_PREFIX_LEN: usize = WELCOME_PREFIX.len();

    pub const INITIATE_PREFIX: &[u8] = b"\x08INITIATE";
    pub const INITIATE_PREFIX_LEN: usize = INITIATE_PREFIX.len();

    pub const READY_PREFIX: &[u8] = b"\x05READY";
    pub const READY_PREFIX_LEN: usize = READY_PREFIX.len();

    pub const ERROR_PREFIX: &[u8] = b"\x05ERROR";
    pub const ERROR_PREFIX_LEN: usize = ERROR_PREFIX.len();

    pub const BRIEF_LEN_SIZE: usize = 1; // size of char in bytes
}
