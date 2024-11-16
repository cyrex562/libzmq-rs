use crate::message::Message;

/// Interface to be implemented by message encoder.
pub trait IEncoder {
    /// The function returns a batch of binary data. The data
    /// are filled to a supplied buffer. If no buffer is supplied
    /// encoder will provide buffer of its own.
    /// Function returns 0 when a new message is required.
    fn encode(&mut self, buffer: Option<&mut [u8]>, size: usize) -> usize;

    /// Load a new message into encoder.
    fn load_msg(&mut self, msg: &mut Message);
}
