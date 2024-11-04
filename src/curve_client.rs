use crate::{
    mechanism::{Mechanism, Status},
    message::Message,
    session::Session,
    options::Options,
};

#[cfg(feature = "curve")]
pub struct CurveClient {
    state: State,
    tools: CurveClientTools,
    session: Session,
    options: Options,
}

#[cfg(feature = "curve")]
#[derive(PartialEq)]
enum State {
    SendHello,
    ExpectWelcome,
    SendInitiate,
    ExpectReady,
    ErrorReceived,
    Connected,
}

#[cfg(feature = "curve")]
impl CurveClient {
    pub fn new(session: Session, options: Options, downgrade_sub: bool) -> Self {
        CurveClient {
            state: State::SendHello,
            tools: CurveClientTools::new(
                options.curve_public_key,
                options.curve_secret_key,
                options.curve_server_key,
            ),
            session,
            options,
        }
    }

    fn produce_hello(&mut self, msg: &mut Message) -> Result<(), Box<dyn std::error::Error>> {
        msg.init_size(200)?;
        self.tools.produce_hello(msg.data_mut(), self.get_and_inc_nonce())?;
        Ok(())
    }

    fn process_welcome(&mut self, msg_data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        self.tools.process_welcome(msg_data, self.get_writable_precom_buffer())?;
        self.state = State::SendInitiate;
        Ok(())
    }

    fn produce_initiate(&mut self, msg: &mut Message) -> Result<(), Box<dyn std::error::Error>> {
        let metadata_length = self.basic_properties_len();
        let mut metadata_plaintext = vec![0u8; metadata_length];
        
        self.add_basic_properties(&mut metadata_plaintext);
        
        let msg_size = 113 + 128 + CRYPTO_BOX_BOXZEROBYTES + metadata_length;
        msg.init_size(msg_size)?;
        
        self.tools.produce_initiate(
            msg.data_mut(),
            msg_size,
            self.get_and_inc_nonce(),
            &metadata_plaintext,
            metadata_length,
        )?;
        
        Ok(())
    }

    fn process_ready(&mut self, msg_data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        if msg_data.len() < 30 {
            return Err("Malformed READY command".into());
        }

        let clen = (msg_data.len() - 14) + CRYPTO_BOX_BOXZEROBYTES;
        let mut ready_nonce = [0u8; CRYPTO_BOX_NONCEBYTES];
        let mut ready_plaintext = vec![0u8; CRYPTO_BOX_ZEROBYTES + clen];
        let mut ready_box = vec![0u8; CRYPTO_BOX_BOXZEROBYTES + 16 + clen];

        // Copy nonce data
        ready_nonce[..16].copy_from_slice(b"CurveZMQREADY---");
        ready_nonce[16..24].copy_from_slice(&msg_data[6..14]);
        
        self.set_peer_nonce(u64::from_le_bytes(msg_data[6..14].try_into()?));

        // Decrypt the message
        crypto_box_open_afternm(
            &mut ready_plaintext,
            &ready_box,
            clen,
            &ready_nonce,
            self.get_precom_buffer(),
        )?;

        self.parse_metadata(&ready_plaintext[CRYPTO_BOX_ZEROBYTES..])?;
        self.state = State::Connected;
        
        Ok(())
    }

    fn process_error(&mut self, msg_data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        if self.state != State::ExpectWelcome && self.state != State::ExpectReady {
            return Err("Unexpected command".into());
        }
        
        if msg_data.len() < 7 {
            return Err("Malformed ERROR command".into());
        }

        let error_reason_len = msg_data[6] as usize;
        if error_reason_len > msg_data.len() - 7 {
            return Err("Malformed ERROR command".into());
        }

        let error_reason = std::str::from_utf8(&msg_data[7..7 + error_reason_len])?;
        self.handle_error_reason(error_reason);
        self.state = State::ErrorReceived;
        
        Ok(())
    }
}

#[cfg(feature = "curve")]
impl Mechanism for CurveClient {
    fn next_handshake_command(&mut self, msg: &mut Message) -> Result<(), Box<dyn std::error::Error>> {
        match self.state {
            State::SendHello => {
                self.produce_hello(msg)?;
                self.state = State::ExpectWelcome;
            }
            State::SendInitiate => {
                self.produce_initiate(msg)?;
                self.state = State::ExpectReady;
            }
            _ => return Err("Invalid state".into()),
        }
        Ok(())
    }

    fn process_handshake_command(&mut self, msg: &mut Message) -> Result<(), Box<dyn std::error::Error>> {
        let msg_data = msg.data();
        
        if CurveClientTools::is_handshake_command_welcome(msg_data) {
            self.process_welcome(msg_data)?;
        } else if CurveClientTools::is_handshake_command_ready(msg_data) {
            self.process_ready(msg_data)?;
        } else if CurveClientTools::is_handshake_command_error(msg_data) {
            self.process_error(msg_data)?;
        } else {
            return Err("Unexpected command".into());
        }

        msg.close()?;
        msg.init()?;
        
        Ok(())
    }

    fn encode(&mut self, msg: &mut Message) -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(self.state, State::Connected);
        self.encode_message(msg)
    }

    fn decode(&mut self, msg: &mut Message) -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(self.state, State::Connected);
        self.decode_message(msg)
    }

    fn status(&self) -> Status {
        match self.state {
            State::Connected => Status::Ready,
            State::ErrorReceived => Status::Error,
            _ => Status::Handshaking,
        }
    }
}