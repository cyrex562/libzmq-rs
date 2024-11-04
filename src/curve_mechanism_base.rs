#[cfg(feature = "curve")]
use std::convert::TryInto;
use std::mem;
use sodiumoxide::crypto::box_;

const FLAG_MASK: u8 = 0x03; // msg_t::more | msg_t::command
const FLAGS_LEN: usize = 1;
const NONCE_PREFIX_LEN: usize = 16;
const MESSAGE_COMMAND: &[u8] = b"\x07MESSAGE";
const MESSAGE_COMMAND_LEN: usize = 8;
const MESSAGE_HEADER_LEN: usize = MESSAGE_COMMAND_LEN + mem::size_of::<u64>();

#[derive(Debug)]
pub struct CurveEncoding {
    encode_nonce_prefix: [u8; NONCE_PREFIX_LEN],
    decode_nonce_prefix: [u8; NONCE_PREFIX_LEN],
    cn_nonce: u64,
    cn_peer_nonce: u64,
    cn_precom: box_::PrecomputedKey,
    downgrade_sub: bool,
}

impl CurveEncoding {
    pub fn new(
        encode_nonce_prefix: &[u8],
        decode_nonce_prefix: &[u8],
        precomputed_key: box_::PrecomputedKey,
        downgrade_sub: bool,
    ) -> Self {
        let mut enc_prefix = [0u8; NONCE_PREFIX_LEN];
        let mut dec_prefix = [0u8; NONCE_PREFIX_LEN];
        enc_prefix.copy_from_slice(&encode_nonce_prefix[..NONCE_PREFIX_LEN]);
        dec_prefix.copy_from_slice(&decode_nonce_prefix[..NONCE_PREFIX_LEN]);
        
        Self {
            encode_nonce_prefix: enc_prefix,
            decode_nonce_prefix: dec_prefix,
            cn_nonce: 1,
            cn_peer_nonce: 1,
            cn_precom: precomputed_key,
            downgrade_sub,
        }
    }

    fn get_and_inc_nonce(&mut self) -> u64 {
        let nonce = self.cn_nonce;
        self.cn_nonce += 1;
        nonce
    }

    fn set_peer_nonce(&mut self, nonce: u64) {
        self.cn_peer_nonce = nonce;
    }

    fn check_validity(&self, msg: &[u8]) -> Result<(), i32> {
        if msg.len() < MESSAGE_COMMAND_LEN || 
           !msg.starts_with(MESSAGE_COMMAND) {
            return Err(ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND);
        }

        if msg.len() < MESSAGE_HEADER_LEN + box_::MACBYTES + FLAGS_LEN {
            return Err(ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_MESSAGE);
        }

        let nonce = u64::from_le_bytes(msg[MESSAGE_COMMAND_LEN..][..8].try_into().unwrap());
        if nonce <= self.cn_peer_nonce {
            return Err(ZMQ_PROTOCOL_ERROR_ZMTP_INVALID_SEQUENCE);
        }
        
        Ok(())
    }

    pub fn encode(&mut self, msg: &[u8], flags: u8) -> Result<Vec<u8>, i32> {
        let mut nonce = vec![0u8; box_::NONCEBYTES];
        nonce[..NONCE_PREFIX_LEN].copy_from_slice(&self.encode_nonce_prefix);
        nonce[NONCE_PREFIX_LEN..NONCE_PREFIX_LEN+8]
            .copy_from_slice(&self.get_and_inc_nonce().to_le_bytes());

        let mut plaintext = Vec::with_capacity(FLAGS_LEN + msg.len());
        plaintext.push(flags & FLAG_MASK);
        plaintext.extend_from_slice(msg);

        let ciphertext = box_::seal_precomputed(&plaintext, 
                                              &box_::Nonce::from_slice(&nonce).unwrap(),
                                              &self.cn_precom);

        let mut result = Vec::with_capacity(MESSAGE_HEADER_LEN + ciphertext.len());
        result.extend_from_slice(MESSAGE_COMMAND);
        result.extend_from_slice(&nonce[NONCE_PREFIX_LEN..NONCE_PREFIX_LEN+8]);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    pub fn decode(&mut self, msg: &[u8]) -> Result<(Vec<u8>, u8), i32> {
        self.check_validity(msg)?;

        let mut nonce = vec![0u8; box_::NONCEBYTES];
        nonce[..NONCE_PREFIX_LEN].copy_from_slice(&self.decode_nonce_prefix);
        nonce[NONCE_PREFIX_LEN..NONCE_PREFIX_LEN+8]
            .copy_from_slice(&msg[MESSAGE_COMMAND_LEN..MESSAGE_COMMAND_LEN+8]);

        let ciphertext = &msg[MESSAGE_HEADER_LEN..];
        
        let plaintext = box_::open_precomputed(
            ciphertext,
            &box_::Nonce::from_slice(&nonce).unwrap(),
            &self.cn_precom
        ).map_err(|_| ZMQ_PROTOCOL_ERROR_ZMTP_CRYPTOGRAPHIC)?;

        let flags = plaintext[0];
        Ok((plaintext[FLAGS_LEN..].to_vec(), flags))
    }
}

// Error codes
const ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND: i32 = -1;
const ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_MESSAGE: i32 = -2;  
const ZMQ_PROTOCOL_ERROR_ZMTP_INVALID_SEQUENCE: i32 = -3;
const ZMQ_PROTOCOL_ERROR_ZMTP_CRYPTOGRAPHIC: i32 = -4;