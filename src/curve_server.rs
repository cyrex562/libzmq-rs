use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::secretbox;

use crate::err::ZmqError;
use crate::message::Message;
use crate::options::Options;
use crate::session_base::SessionBase;

// Constants mapped from C++
const CRYPTO_BOX_PUBLICKEYBYTES: usize = 32;
const CRYPTO_BOX_SECRETKEYBYTES: usize = 32;
const CRYPTO_BOX_NONCEBYTES: usize = 24;
const CRYPTO_BOX_ZEROBYTES: usize = 32;
const CRYPTO_BOX_BOXZEROBYTES: usize = 16;

#[derive(Debug)]
enum State {
    WaitingForHello,
    SendingWelcome,
    WaitingForInitiate,
    SendingReady,
    Ready,
    SendingError,
    ErrorSent,
    WaitingForZapReply,
}

pub struct CurveServer {
    state: State,
    secret_key: [u8; CRYPTO_BOX_SECRETKEYBYTES],
    cn_public: [u8; CRYPTO_BOX_PUBLICKEYBYTES],
    cn_secret: [u8; CRYPTO_BOX_SECRETKEYBYTES],
    cn_client: [u8; CRYPTO_BOX_PUBLICKEYBYTES],
    cookie_key: [u8; secretbox::KEYBYTES],
    precom_buffer: box_::PrecomputedKey,
    peer_address: String,
    status_code: String,
    nonce_counter: u64,
}

impl CurveServer {
    pub fn new(
        session: &SessionBase,
        peer_address: String,
        options: &Options,
        downgrade_sub: bool,
    ) -> Self {
        let secret_key = options.curve_secret_key;
        let (public_key, secret_key) = box_::gen_keypair();

        CurveServer {
            state: State::WaitingForHello,
            secret_key,
            cn_public: public_key.0,
            cn_secret: secret_key.0,
            cn_client: [0; CRYPTO_BOX_PUBLICKEYBYTES],
            cookie_key: [0; secretbox::KEYBYTES],
            precom_buffer: box_::PrecomputedKey([0; box_::PRECOMPUTEDKEYBYTES]),
            peer_address,
            status_code: String::new(),
            nonce_counter: 0,
        }
    }

    pub fn next_handshake_command(&mut self, msg: &mut Message) -> Result<(), ZmqError> {
        match self.state {
            State::SendingWelcome => {
                self.produce_welcome(msg)?;
                self.state = State::WaitingForInitiate;
                Ok(())
            }
            State::SendingReady => {
                self.produce_ready(msg)?;
                self.state = State::Ready;
                Ok(())
            }
            State::SendingError => {
                self.produce_error(msg)?;
                self.state = State::ErrorSent;
                Ok(())
            }
            _ => Err(ZmqError::WouldBlock),
        }
    }

    pub fn process_handshake_command(&mut self, msg: &Message) -> Result<(), ZmqError> {
        match self.state {
            State::WaitingForHello => self.process_hello(msg),
            State::WaitingForInitiate => self.process_initiate(msg),
            _ => Err(ZmqError::Protocol("Invalid handshake command")),
        }
    }

    fn process_hello(&mut self, msg: &Message) -> Result<(), ZmqError> {
        let data = msg.data();
        if data.len() != 200 || !data.starts_with(b"\x05HELLO") {
            return Err(ZmqError::Protocol("Invalid HELLO message"));
        }

        // Version checking
        if data[6] != 1 || data[7] != 0 {
            return Err(ZmqError::Protocol("Unknown version number"));
        }

        // Save client's public key
        self.cn_client.copy_from_slice(&data[80..112]);

        // Process crypto box
        let nonce = create_nonce("CurveZMQHELLO---", &data[112..120])?;
        let plaintext = box_::open(
            &data[120..],
            &nonce,
            &box_::PublicKey(self.cn_client),
            &box_::SecretKey(self.secret_key),
        )?;

        self.state = State::SendingWelcome;
        Ok(())
    }

    // ... Additional method implementations for produce_welcome, process_initiate, etc.
    // would follow similar patterns of converting the C++ logic to Rust
}

// Helper functions
fn create_nonce(prefix: &str, suffix: &[u8]) -> Result<box_::Nonce, ZmqError> {
    let mut nonce = [0u8; CRYPTO_BOX_NONCEBYTES];
    nonce[..prefix.len()].copy_from_slice(prefix.as_bytes());
    nonce[prefix.len()..prefix.len() + suffix.len()].copy_from_slice(suffix);
    Ok(box_::Nonce(nonce))
}
