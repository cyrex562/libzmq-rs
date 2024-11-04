use crate::command_t;

/// Interface to be implemented by mailbox.
pub trait IMailbox {
    /// Send a command to the mailbox
    fn send(&mut self, cmd: &command_t);
    
    /// Receive a command from the mailbox with timeout
    /// Returns Ok(command) on success, or an error
    fn recv(&mut self, timeout: i32) -> Result<command_t, Box<dyn std::error::Error>>;

    /// Close file descriptors in the signaller when forked
    #[cfg(target_family = "unix")]
    fn forked(&mut self);
}
