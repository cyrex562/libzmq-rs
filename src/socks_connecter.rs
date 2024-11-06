use std::net::{TcpStream, SocketAddr, ToSocketAddrs};
use std::io::{Read, Write, Error, ErrorKind};
use std::time::Duration;

#[derive(Debug)]
enum AuthMethod {
    NoAuthRequired = 0x00,
    BasicAuth = 0x02,
    NoAcceptableMethod = 0xff,
}

#[derive(Debug)]
enum Status {
    Unplugged,
    WaitingForReconnect,
    WaitingForProxyConnection,
    SendingGreeting,
    WaitingForChoice,
    SendingBasicAuthRequest,
    WaitingForAuthResponse,
    SendingRequest,
    WaitingForResponse,
}

pub struct SocksConnector {
    stream: Option<TcpStream>,
    proxy_addr: String,
    target_addr: String,
    auth_method: AuthMethod,
    auth_username: String,
    auth_password: String,
    status: Status,
}

impl SocksConnector {
    pub fn new(proxy_addr: String, target_addr: String) -> Self {
        SocksConnector {
            stream: None,
            proxy_addr,
            target_addr,
            auth_method: AuthMethod::NoAuthRequired,
            auth_username: String::new(),
            auth_password: String::new(),
            status: Status::Unplugged,
        }
    }

    pub fn set_auth_method_basic(&mut self, username: String, password: String) {
        self.auth_method = AuthMethod::BasicAuth;
        self.auth_username = username;
        self.auth_password = password;
    }

    pub fn set_auth_method_none(&mut self) {
        self.auth_method = AuthMethod::NoAuthRequired;
        self.auth_username.clear();
        self.auth_password.clear();
    }

    pub fn connect(&mut self) -> Result<TcpStream, Error> {
        // Connect to proxy
        let stream = TcpStream::connect(&self.proxy_addr)?;
        stream.set_nonblocking(true)?;
        
        // Send SOCKS5 greeting
        let mut greeting = vec![5, 1, self.auth_method as u8];
        stream.write_all(&greeting)?;
        
        // Read server choice
        let mut response = [0u8; 2];
        stream.read_exact(&mut response)?;
        
        if response[0] != 5 {
            return Err(Error::new(ErrorKind::Other, "Invalid SOCKS protocol"));
        }

        match response[1] {
            0x00 => self.handle_no_auth(stream),
            0x02 => self.handle_basic_auth(stream),
            _ => Err(Error::new(ErrorKind::Other, "Unsupported auth method")),
        }
    }

    fn handle_no_auth(mut self, mut stream: TcpStream) -> Result<TcpStream, Error> {
        self.send_connection_request(&mut stream)?;
        self.receive_connection_response(&mut stream)
    }

    fn handle_basic_auth(mut self, mut stream: TcpStream) -> Result<TcpStream, Error> {
        // Send username/password auth request
        let mut auth_request = vec![1];
        auth_request.push(self.auth_username.len() as u8);
        auth_request.extend(self.auth_username.as_bytes());
        auth_request.push(self.auth_password.len() as u8);
        auth_request.extend(self.auth_password.as_bytes());
        
        stream.write_all(&auth_request)?;

        // Read auth response
        let mut response = [0u8; 2];
        stream.read_exact(&mut response)?;

        if response[1] != 0 {
            return Err(Error::new(ErrorKind::Other, "Authentication failed"));
        }

        self.send_connection_request(&mut stream)?;
        self.receive_connection_response(&mut stream)
    }

    fn send_connection_request(&self, stream: &mut TcpStream) -> Result<(), Error> {
        // Parse target address
        let addr = self.target_addr.to_socket_addrs()?.next()
            .ok_or_else(|| Error::new(ErrorKind::Other, "Invalid target address"))?;
        
        let mut request = vec![
            5, // SOCKS version
            1, // CONNECT command
            0, // Reserved
        ];

        match addr {
            SocketAddr::V4(addr) => {
                request.push(1); // IPv4
                request.extend_from_slice(&addr.ip().octets());
                request.extend_from_slice(&addr.port().to_be_bytes());
            }
            SocketAddr::V6(addr) => {
                request.push(4); // IPv6
                request.extend_from_slice(&addr.ip().octets());
                request.extend_from_slice(&addr.port().to_be_bytes());
            }
        }

        stream.write_all(&request)?;
        Ok(())
    }

    fn receive_connection_response(&self, stream: &mut TcpStream) -> Result<TcpStream, Error> {
        let mut response = [0u8; 4];
        stream.read_exact(&mut response)?;

        if response[1] != 0 {
            return Err(Error::new(ErrorKind::Other, "Connection failed"));
        }

        // Skip the rest of the response based on address type
        match response[3] {
            1 => { let mut addr = [0u8; 6]; stream.read_exact(&mut addr)?; }
            4 => { let mut addr = [0u8; 18]; stream.read_exact(&mut addr)?; }
            _ => return Err(Error::new(ErrorKind::Other, "Invalid address type")),
        }

        stream.set_nonblocking(false)?;
        Ok(stream)
    }
}
