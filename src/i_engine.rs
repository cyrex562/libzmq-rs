use std::fmt;

// Assuming these types exist in other modules
pub struct IoThread;
pub struct SessionBase;
pub struct EndpointUriPair;

#[derive(Debug)]
pub enum ErrorReason {
    ProtocolError,
    ConnectionError,
    TimeoutError,
}

pub trait IEngine {
    /// Indicate if the engine has a handshake stage.
    /// If engine has handshake stage, engine must call session.engine_ready when the handshake is complete.
    fn has_handshake_stage(&self) -> bool;

    /// Plug the engine to the session.
    fn plug(&mut self, io_thread: &mut IoThread, session: &mut SessionBase);

    /// Terminate and deallocate the engine.
    /// Note that 'detached' events are not fired on termination.
    fn terminate(&mut self);

    /// This method is called by the session to signalize that more
    /// messages can be written to the pipe.
    /// Returns false if the engine was deleted due to an error.
    fn restart_input(&mut self) -> bool;

    /// This method is called by the session to signalize that there
    /// are messages to send available.
    fn restart_output(&mut self);

    fn zap_msg_available(&mut self);

    fn get_endpoint(&self) -> &EndpointUriPair;
}
