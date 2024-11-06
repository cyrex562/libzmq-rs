use std::net::{IpAddr, SocketAddr};
use std::io::{Read, Write, Result};

const MAX_UINT8: usize = u8::MAX as usize;

pub struct SocksGreeting {
    methods: Vec<u8>,
}

impl SocksGreeting {
    pub fn new(method: u8) -> Self {
        Self { methods: vec![method] }
    }

    pub fn with_methods(methods: &[u8]) -> Self {
        Self { methods: methods.to_vec() }
    }
}

pub struct SocksGreetingEncoder {
    buffer: Vec<u8>,
    bytes_written: usize,
}

impl SocksGreetingEncoder {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            bytes_written: 0,
        }
    }

    pub fn encode(&mut self, greeting: &SocksGreeting) {
        self.buffer.clear();
        self.buffer.push(0x05);
        self.buffer.push(greeting.methods.len() as u8);
        self.buffer.extend_from_slice(&greeting.methods);
        self.bytes_written = 0;
    }

    pub fn write<W: Write>(&mut self, writer: &mut W) -> Result<usize> {
        let remaining = &self.buffer[self.bytes_written..];
        let bytes = writer.write(remaining)?;
        self.bytes_written += bytes;
        Ok(bytes)
    }

    pub fn has_pending_data(&self) -> bool {
        self.bytes_written < self.buffer.len()
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
        self.bytes_written = 0;
    }
}

pub struct SocksChoice {
    pub method: u8,
}

pub struct SocksChoiceDecoder {
    buffer: [u8; 2],
    bytes_read: usize,
}

impl SocksChoiceDecoder {
    pub fn new() -> Self {
        Self {
            buffer: [0; 2],
            bytes_read: 0,
        }
    }

    pub fn read<R: Read>(&mut self, reader: &mut R) -> Result<usize> {
        let remaining = &mut self.buffer[self.bytes_read..];
        let bytes = reader.read(remaining)?;
        if bytes > 0 && self.buffer[0] != 0x05 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid SOCKS version",
            ));
        }
        self.bytes_read += bytes;
        Ok(bytes)
    }

    pub fn message_ready(&self) -> bool {
        self.bytes_read == 2
    }

    pub fn decode(&self) -> SocksChoice {
        assert!(self.message_ready());
        SocksChoice {
            method: self.buffer[1]
        }
    }

    pub fn reset(&mut self) {
        self.bytes_read = 0;
    }
}

pub struct SocksBasicAuthRequest {
    username: String,
    password: String,
}

impl SocksBasicAuthRequest {
    pub fn new(username: String, password: String) -> Self {
        assert!(username.len() <= MAX_UINT8);
        assert!(password.len() <= MAX_UINT8);
        Self { username, password }
    }
}

pub struct SocksBasicAuthRequestEncoder {
    buffer: Vec<u8>,
    bytes_written: usize,
}

impl SocksBasicAuthRequestEncoder {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            bytes_written: 0,
        }
    }

    pub fn encode(&mut self, req: &SocksBasicAuthRequest) {
        self.buffer.clear();
        self.buffer.push(0x01);
        self.buffer.push(req.username.len() as u8);
        self.buffer.extend_from_slice(req.username.as_bytes());
        self.buffer.push(req.password.len() as u8);
        self.buffer.extend_from_slice(req.password.as_bytes());
        self.bytes_written = 0;
    }

    // ... rest of implementation similar to SocksGreetingEncoder
}

// Additional structures and implementations follow similar patterns:
// - SocksAuthResponse
// - SocksAuthResponseDecoder
// - SocksRequest
// - SocksRequestEncoder
// - SocksResponse
// - SocksResponseDecoder

// Example of the request/response structures:

pub struct SocksRequest {
    command: u8,
    hostname: String,
    port: u16,
}

impl SocksRequest {
    pub fn new(command: u8, hostname: String, port: u16) -> Self {
        assert!(hostname.len() <= MAX_UINT8);
        Self { command, hostname, port }
    }
}

pub struct SocksResponse {
    response_code: u8,
    address: String,
    port: u16,
}

impl SocksResponse {
    pub fn new(response_code: u8, address: String, port: u16) -> Self {
        Self { response_code, address, port }
    }
}
