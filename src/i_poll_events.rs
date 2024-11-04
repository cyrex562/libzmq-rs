//! Poll events interface for ZMQ implementation
//! Converted from original C++ code

/// Trait to be implemented by objects that want to be notified
/// about events on file descriptors.
pub trait IPollEvents {
    /// Called by I/O thread when file descriptor is ready for reading.
    fn in_event(&mut self);

    /// Called by I/O thread when file descriptor is ready for writing.
    fn out_event(&mut self);

    /// Called when timer expires.
    fn timer_event(&mut self, id: i32);
}
