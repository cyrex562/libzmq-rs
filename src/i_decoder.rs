use std::error::Error;

// Message type placeholder - would need to be defined based on msg_t implementation
pub struct Message;

pub trait Decoder {
    // Get mutable access to internal buffer
    fn get_buffer(&mut self) -> (&mut [u8], usize);

    // Resize the internal buffer
    fn resize_buffer(&mut self, new_size: usize);

    // Decode data from buffer, returns processed size and decoding result
    fn decode(&mut self, data: &[u8]) -> Result<(bool, usize), Box<dyn Error>>;

    // Get reference to the decoded message
    fn msg(&self) -> &Message;
}
