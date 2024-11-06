use std::string::String;

// State enum to replace the implicit state machine
#[derive(PartialEq)]
enum State {
    SendingWelcome,
    WaitingForHello,
    WaitingForInitiate,
    SendingReady,
    SendingError,
    Ready,
    ErrorSent,
    WaitingForZapReply,
}

// Constants for message prefixes
const HELLO_PREFIX: &[u8] = b"HELLO";
const WELCOME_PREFIX: &[u8] = b"WELCOME";
const INITIATE_PREFIX: &[u8] = b"INITIATE";
const READY_PREFIX: &[u8] = b"READY";
const ERROR_PREFIX: &[u8] = b"ERROR";

pub struct PlainServer {
    state: State,
    session: SessionBase,
    peer_address: String,
    options: Options,
    status_code: String,
}

impl PlainServer {
    pub fn new(session: SessionBase, peer_address: String, options: Options) -> Self {
        // Assert ZAP requirements if enforce_domain is set
        if options.zap_enforce_domain {
            assert!(Self::zap_required(&options));
        }

        PlainServer {
            state: State::SendingWelcome,
            session,
            peer_address,
            options,
            status_code: String::new(),
        }
    }

    pub fn next_handshake_command(&mut self, msg: &mut Message) -> Result<(), ZmqError> {
        match self.state {
            State::SendingWelcome => {
                self.produce_welcome(msg);
                self.state = State::WaitingForInitiate;
                Ok(())
            }
            State::SendingReady => {
                self.produce_ready(msg);
                self.state = State::Ready;
                Ok(())
            }
            State::SendingError => {
                self.produce_error(msg);
                self.state = State::ErrorSent;
                Ok(())
            }
            _ => Err(ZmqError::Again),
        }
    }

    pub fn process_handshake_command(&mut self, msg: &mut Message) -> Result<(), ZmqError> {
        match self.state {
            State::WaitingForHello => self.process_hello(msg),
            State::WaitingForInitiate => self.process_initiate(msg),
            _ => {
                self.session.get_socket().event_handshake_failed_protocol(
                    &self.session.get_endpoint(),
                    ZMQ_PROTOCOL_ERROR_ZMTP_UNSPECIFIED,
                );
                Err(ZmqError::Protocol)
            }
        }?;

        msg.close()?;
        msg.init()?;
        Ok(())
    }

    fn process_hello(&mut self, msg: &Message) -> Result<(), ZmqError> {
        self.check_basic_command_structure(msg)?;

        let data = msg.data();
        if data.len() < HELLO_PREFIX.len() || !data.starts_with(HELLO_PREFIX) {
            self.session.get_socket().event_handshake_failed_protocol(
                &self.session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND,
            );
            return Err(ZmqError::Protocol);
        }

        let mut pos = HELLO_PREFIX.len();
        
        // Extract username
        if pos >= data.len() {
            self.session.get_socket().event_handshake_failed_protocol(
                &self.session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_HELLO,
            );
            return Err(ZmqError::Protocol);
        }
        
        let username_len = data[pos] as usize;
        pos += 1;
        
        if pos + username_len > data.len() {
            self.session.get_socket().event_handshake_failed_protocol(
                &self.session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_HELLO,
            );
            return Err(ZmqError::Protocol);
        }
        
        let username = String::from_utf8_lossy(&data[pos..pos + username_len]).to_string();
        pos += username_len;

        // Extract password
        if pos >= data.len() {
            self.session.get_socket().event_handshake_failed_protocol(
                &self.session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_HELLO,
            );
            return Err(ZmqError::Protocol);
        }

        let password_len = data[pos] as usize;
        pos += 1;

        if pos + password_len != data.len() {
            self.session.get_socket().event_handshake_failed_protocol(
                &self.session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_HELLO,
            );
            return Err(ZmqError::Protocol);
        }

        let password = String::from_utf8_lossy(&data[pos..pos + password_len]).to_string();

        self.session.zap_connect()?;
        self.send_zap_request(&username, &password);
        self.state = State::WaitingForZapReply;

        self.receive_and_process_zap_reply()
    }

    fn produce_welcome(&self, msg: &mut Message) {
        msg.init_size(WELCOME_PREFIX.len())
            .expect("Failed to initialize welcome message");
        msg.copy_from_slice(WELCOME_PREFIX);
    }

    fn produce_ready(&self, msg: &mut Message) {
        self.make_command_with_basic_properties(msg, READY_PREFIX);
    }

    fn produce_error(&self, msg: &mut Message) {
        const EXPECTED_STATUS_CODE_LEN: u8 = 3;
        assert_eq!(self.status_code.len(), EXPECTED_STATUS_CODE_LEN as usize);

        msg.init_size(ERROR_PREFIX.len() + 1 + EXPECTED_STATUS_CODE_LEN as usize)
            .expect("Failed to initialize error message");
        
        let mut data = Vec::with_capacity(ERROR_PREFIX.len() + 1 + EXPECTED_STATUS_CODE_LEN as usize);
        data.extend_from_slice(ERROR_PREFIX);
        data.push(EXPECTED_STATUS_CODE_LEN);
        data.extend_from_slice(self.status_code.as_bytes());
        
        msg.copy_from_slice(&data);
    }

    fn send_zap_request(&self, username: &str, password: &str) {
        let credentials = [username.as_bytes(), password.as_bytes()];
        let credentials_sizes = [username.len(), password.len()];
        
        self.zap_client_send_request(
            "PLAIN",
            &credentials,
            &credentials_sizes,
        );
    }

    // Helper functions would be implemented here...
}
