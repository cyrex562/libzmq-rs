use crate::command::Command;

/// Interface to be implemented by mailbox.
pub trait IMailbox {
    /// Send a command to the mailbox
    fn send(&mut self, cmd: &Command);

    /// Receive a command from the mailbox with timeout
    /// Returns Ok(command) on success, or an error
    fn recv(&mut self, timeout: i32) -> Result<Command, Box<dyn std::error::Error>>;

    /// Close file descriptors in the signaller when forked
    #[cfg(target_family = "unix")]
    fn forked(&mut self);
}
