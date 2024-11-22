#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

use std::ffi::{c_void, CString};
use std::os::raw::{c_char, c_uint};
use std::ptr;

// FFI bindings for NORM C API
#[allow(non_camel_case_types)]
type NormInstanceHandle = *mut c_void;
type NormSessionHandle = *mut c_void;
type NormObjectHandle = *mut c_void;
type NormDescriptor = c_uint;
type NormNodeId = c_uint;
type NormSessionId = c_uint;

const NORM_INSTANCE_INVALID: NormInstanceHandle = ptr::null_mut();
const NORM_SESSION_INVALID: NormSessionHandle = ptr::null_mut();
const NORM_OBJECT_INVALID: NormObjectHandle = ptr::null_mut();
const NORM_NODE_ANY: NormNodeId = 0;
const BUFFER_SIZE: usize = 2048;

// Rust version of NormRxStreamState
#[derive(Debug)]
struct NormRxStreamState {
    norm_stream: NormObjectHandle,
    max_msg_size: i64,
    zero_copy: bool,
    in_batch_size: i32,
    in_sync: bool,
    rx_ready: bool,
    skip_norm_sync: bool,
    buffer: Vec<u8>,
    buffer_count: usize,
}

impl NormRxStreamState {
    fn new(
        norm_stream: NormObjectHandle,
        max_msg_size: i64,
        zero_copy: bool,
        in_batch_size: i32,
    ) -> Self {
        Self {
            norm_stream,
            max_msg_size,
            zero_copy,
            in_batch_size,
            in_sync: false,
            rx_ready: false,
            skip_norm_sync: false,
            buffer: Vec::with_capacity(BUFFER_SIZE),
            buffer_count: 0,
        }
    }

    fn init(&mut self) -> bool {
        self.in_sync = false;
        self.skip_norm_sync = false;
        self.buffer_count = 0;
        self.buffer.clear();
        true
    }

    fn decode(&mut self) -> i32 {
        // Simplified decoder that needs to be expanded based on v2_decoder implementation
        if self.buffer_count == 0 {
            return 0; // need more data
        }

        if self.skip_norm_sync {
            self.buffer_count -= 1;
            self.skip_norm_sync = false;
        }

        // Simplified decoding - in reality would need full ZMQ message decoding
        1 // Indicate message completion for now
    }
}

// Main NORM engine implementation
pub struct NormEngine {
    norm_instance: NormInstanceHandle,
    norm_session: NormSessionHandle,
    norm_tx_stream: NormObjectHandle,
    is_sender: bool,
    is_receiver: bool,
    tx_buffer: [u8; BUFFER_SIZE],
    tx_len: usize,
    tx_index: usize,
    rx_states: Vec<NormRxStreamState>,
}

impl NormEngine {
    pub fn new() -> Self {
        Self {
            norm_instance: NORM_INSTANCE_INVALID,
            norm_session: NORM_SESSION_INVALID,
            norm_tx_stream: NORM_OBJECT_INVALID,
            is_sender: false,
            is_receiver: false,
            tx_buffer: [0; BUFFER_SIZE],
            tx_len: 0,
            tx_index: 0,
            rx_states: Vec::new(),
        }
    }

    pub fn init(&mut self, network: &str, send: bool, recv: bool) -> Result<(), i32> {
        // Parse network string
        let parts: Vec<&str> = network.split(',').collect();
        let (local_id, addr_part) = match parts.len() {
            1 => (NORM_NODE_ANY, parts[0]),
            2 => {
                let id = parts[0].parse().unwrap_or(NORM_NODE_ANY);
                (id, parts[1])
            }
            _ => return Err(-1),
        };

        // Create NORM instance
        unsafe {
            self.norm_instance = norm_create_instance();
            if self.norm_instance == NORM_INSTANCE_INVALID {
                return Err(-1);
            }

            // Parse address and port
            let addr_parts: Vec<&str> = addr_part.split(':').collect();
            if addr_parts.len() != 2 {
                return Err(-1);
            }

            let addr = CString::new(addr_parts[0]).unwrap();
            let port: u16 = addr_parts[1].parse().unwrap_or(0);

            self.norm_session =
                norm_create_session(self.norm_instance, addr.as_ptr(), port, local_id);

            if self.norm_session == NORM_SESSION_INVALID {
                norm_destroy_instance(self.norm_instance);
                self.norm_instance = NORM_INSTANCE_INVALID;
                return Err(-1);
            }
        }

        self.is_sender = send;
        self.is_receiver = recv;

        Ok(())
    }

    pub fn shutdown(&mut self) {
        unsafe {
            if self.is_receiver {
                norm_stop_receiver(self.norm_session);
                self.rx_states.clear();
            }

            if self.is_sender {
                norm_stop_sender(self.norm_session);
            }

            if self.norm_session != NORM_SESSION_INVALID {
                norm_destroy_session(self.norm_session);
                self.norm_session = NORM_SESSION_INVALID;
            }

            if self.norm_instance != NORM_INSTANCE_INVALID {
                norm_stop_instance(self.norm_instance);
                norm_destroy_instance(self.norm_instance);
                self.norm_instance = NORM_INSTANCE_INVALID;
            }
        }
    }
}

// FFI declarations for NORM C API
extern "C" {
    fn norm_create_instance() -> NormInstanceHandle;
    fn norm_destroy_instance(instance: NormInstanceHandle);
    fn norm_stop_instance(instance: NormInstanceHandle);
    fn norm_create_session(
        instance: NormInstanceHandle,
        addr: *const c_char,
        port: u16,
        local_id: NormNodeId,
    ) -> NormSessionHandle;
    fn norm_destroy_session(session: NormSessionHandle);
    fn norm_stop_sender(session: NormSessionHandle);
    fn norm_stop_receiver(session: NormSessionHandle);
}

impl Drop for NormEngine {
    fn drop(&mut self) {
        self.shutdown();
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_norm_engine_create() {
        let engine = NormEngine::new();
        assert_eq!(engine.norm_instance, NORM_INSTANCE_INVALID);
        assert_eq!(engine.norm_session, NORM_SESSION_INVALID);
    }

    #[test]
    fn test_norm_engine_init() {
        let mut engine = NormEngine::new();
        let result = engine.init("5000,127.0.0.1:6003", true, true);
        assert!(result.is_ok());
        engine.shutdown();
    }
}
