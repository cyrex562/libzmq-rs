
// Definition of constants for ZMTP/2.0 transport protocol
pub mod v2_protocol {
    // Message flags
    pub const MORE_FLAG: u8 = 1;
    pub const LARGE_FLAG: u8 = 2;
    pub const COMMAND_FLAG: u8 = 4;
}
