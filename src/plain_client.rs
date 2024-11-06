use std::mem;

// Constants from plain_common
const WELCOME_PREFIX: &[u8] = b"WELCOME";
const WELCOME_PREFIX_LEN: usize = WELCOME_PREFIX.len();
const READY_PREFIX: &[u8] = b"READY";
const READY_PREFIX_LEN: usize = READY_PREFIX.len();
const ERROR_PREFIX: &[u8] = b"ERROR";
const ERROR_PREFIX_LEN: usize = ERROR_PREFIX.len();
const HELLO_PREFIX: &[u8] = b"HELLO";
const HELLO_PREFIX_LEN: usize = HELLO_PREFIX.len();
const INITIATE_PREFIX: &[u8] = b"INITIATE";
const INITIATE_PREFIX_LEN: usize = INITIATE_PREFIX.len();
const BRIEF_LEN_SIZE: usize = 1;

#[derive(PartialEq)]
enum State {
    SendingHello,
    WaitingForWelcome,
    SendingInitiate,
    WaitingForReady,
    ErrorCommandReceived,
    Ready,
}

pub struct PlainClient {
    state: State,
    options: Options,
    session: SessionBase,
}

impl PlainClient {
    pub fn new(session: SessionBase, options: Options) -> Self {
        PlainClient {
            state: State::SendingHello,
            options,
            session,
        }
    }

    pub fn next_handshake_command(&mut self, msg: &mut Message) -> Result<(), i32> {
        match self.state {
            State::SendingHello => {
                self.produce_hello(msg)?;
                self.state = State::WaitingForWelcome;
                Ok(())
            }
            State::SendingInitiate => {
                self.produce_initiate(msg)?;
                self.state = State::WaitingForReady;
                Ok(())
            }
            _ => Err(libc::EAGAIN),
        }
    }

    pub fn process_handshake_command(&mut self, msg: &mut Message) -> Result<(), i32> {
        let data = msg.data();
        
        let result = if data.len() >= WELCOME_PREFIX_LEN && 
                       data.starts_with(WELCOME_PREFIX) {
            self.process_welcome(data)
        } else if data.len() >= READY_PREFIX_LEN && 
                  data.starts_with(READY_PREFIX) {
            self.process_ready(&data[READY_PREFIX_LEN..])
        } else if data.len() >= ERROR_PREFIX_LEN && 
                  data.starts_with(ERROR_PREFIX) {
            self.process_error(&data[ERROR_PREFIX_LEN..])
        } else {
            self.session.get_socket().event_handshake_failed_protocol(
                self.session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND
            );
            Err(libc::EPROTO)
        };

        if result.is_ok() {
            msg.close()?;
            msg.init()?;
        }

        result
    }

    pub fn status(&self) -> Status {
        match self.state {
            State::Ready => Status::Ready,
            State::ErrorCommandReceived => Status::Error,
            _ => Status::Handshaking,
        }
    }

    fn produce_hello(&self, msg: &mut Message) -> Result<(), i32> {
        let username = &self.options.plain_username;
        let password = &self.options.plain_password;

        assert!(username.len() <= u8::MAX as usize);
        assert!(password.len() <= u8::MAX as usize);

        let command_size = HELLO_PREFIX_LEN + BRIEF_LEN_SIZE + username.len() +
                         BRIEF_LEN_SIZE + password.len();

        msg.init_size(command_size)?;
        let mut data = Vec::with_capacity(command_size);
        
        data.extend_from_slice(HELLO_PREFIX);
        data.push(username.len() as u8);
        data.extend_from_slice(username.as_bytes());
        data.push(password.len() as u8);
        data.extend_from_slice(password.as_bytes());
        
        msg.set_data(data);
        Ok(())
    }

    fn produce_initiate(&self, msg: &mut Message) -> Result<(), i32> {
        self.make_command_with_basic_properties(msg, INITIATE_PREFIX)
    }

    fn process_welcome(&mut self, data: &[u8]) -> Result<(), i32> {
        if self.state != State::WaitingForWelcome {
            self.session.get_socket().event_handshake_failed_protocol(
                self.session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND
            );
            return Err(libc::EPROTO);
        }

        if data.len() != WELCOME_PREFIX_LEN {
            self.session.get_socket().event_handshake_failed_protocol(
                self.session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_WELCOME
            );
            return Err(libc::EPROTO);
        }

        self.state = State::SendingInitiate;
        Ok(())
    }

    fn process_ready(&mut self, data: &[u8]) -> Result<(), i32> {
        if self.state != State::WaitingForReady {
            self.session.get_socket().event_handshake_failed_protocol(
                self.session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND
            );
            return Err(libc::EPROTO);
        }

        match self.parse_metadata(data) {
            Ok(_) => {
                self.state = State::Ready;
                Ok(())
            }
            Err(e) => {
                self.session.get_socket().event_handshake_failed_protocol(
                    self.session.get_endpoint(),
                    ZMQ_PROTOCOL_ERROR_ZMTP_INVALID_METADATA
                );
                Err(e)
            }
        }
    }

    fn process_error(&mut self, data: &[u8]) -> Result<(), i32> {
        if self.state != State::WaitingForWelcome && self.state != State::WaitingForReady {
            self.session.get_socket().event_handshake_failed_protocol(
                self.session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND
            );
            return Err(libc::EPROTO);
        }

        if data.len() < BRIEF_LEN_SIZE {
            self.session.get_socket().event_handshake_failed_protocol(
                self.session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_ERROR
            );
            return Err(libc::EPROTO);
        }

        let error_reason_len = data[0] as usize;
        if error_reason_len > data.len() - BRIEF_LEN_SIZE {
            self.session.get_socket().event_handshake_failed_protocol(
                self.session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_ERROR
            );
            return Err(libc::EPROTO);
        }

        let error_reason = &data[BRIEF_LEN_SIZE..BRIEF_LEN_SIZE + error_reason_len];
        self.handle_error_reason(error_reason)?;
        
        self.state = State::ErrorCommandReceived;
        Ok(())
    }
}
