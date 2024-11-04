#![allow(non_snake_case)]

use sodiumoxide::crypto::box_;
use std::convert::TryInto;

const CRYPTO_BOX_NONCEBYTES: usize = 24;
const CRYPTO_BOX_PUBLICKEYBYTES: usize = 32;
const CRYPTO_BOX_SECRETKEYBYTES: usize = 32;
const CRYPTO_BOX_ZEROBYTES: usize = 32;
const CRYPTO_BOX_BOXZEROBYTES: usize = 16;

pub struct CurveClientTools {
    // Our public key (C)
    public_key: [u8; CRYPTO_BOX_PUBLICKEYBYTES],
    // Our secret key (c)
    secret_key: [u8; CRYPTO_BOX_SECRETKEYBYTES],
    // Our short-term public key (C')
    cn_public: [u8; CRYPTO_BOX_PUBLICKEYBYTES],
    // Our short-term secret key (c')
    cn_secret: [u8; CRYPTO_BOX_SECRETKEYBYTES],
    // Server's public key (S)
    server_key: [u8; CRYPTO_BOX_PUBLICKEYBYTES],
    // Server's short-term public key (S')
    cn_server: [u8; CRYPTO_BOX_PUBLICKEYBYTES],
    // Cookie received from server
    cn_cookie: [u8; 96], // 16 + 80
}

impl CurveClientTools {
    pub fn new(
        curve_public_key: &[u8; CRYPTO_BOX_PUBLICKEYBYTES],
        curve_secret_key: &[u8; CRYPTO_BOX_SECRETKEYBYTES],
        curve_server_key: &[u8; CRYPTO_BOX_PUBLICKEYBYTES],
    ) -> Self {
        let (cn_public, cn_secret) = box_::gen_keypair();
        
        Self {
            public_key: *curve_public_key,
            secret_key: *curve_secret_key,
            cn_public: cn_public.0,
            cn_secret: cn_secret.0,
            server_key: *curve_server_key,
            cn_server: [0; CRYPTO_BOX_PUBLICKEYBYTES],
            cn_cookie: [0; 96],
        }
    }

    pub fn produce_hello(&self, cn_nonce: u64) -> Result<Vec<u8>, &'static str> {
        let mut hello_nonce = [0u8; CRYPTO_BOX_NONCEBYTES];
        hello_nonce[..16].copy_from_slice(b"CurveZMQHELLO---");
        hello_nonce[16..24].copy_from_slice(&cn_nonce.to_le_bytes());

        let hello_plaintext = vec![0u8; CRYPTO_BOX_ZEROBYTES + 64];
        
        let server_pk = box_::PublicKey(self.server_key);
        let cn_sk = box_::SecretKey(self.cn_secret);
        
        let hello_box = box_::seal(
            &hello_plaintext,
            &box_::Nonce(hello_nonce),
            &server_pk,
            &cn_sk,
        );

        let mut hello = Vec::with_capacity(200);
        hello.extend_from_slice(b"\x05HELLO\x01\x00");
        hello.extend_from_slice(&vec![0; 72]); // Anti-amplification padding
        hello.extend_from_slice(&self.cn_public);
        hello.extend_from_slice(&cn_nonce.to_le_bytes());
        hello.extend_from_slice(&hello_box[CRYPTO_BOX_BOXZEROBYTES..]);

        Ok(hello)
    }

    pub fn process_welcome(
        &mut self,
        msg_data: &[u8],
        cn_precom: &mut [u8],
    ) -> Result<(), &'static str> {
        if msg_data.len() != 168 {
            return Err("Invalid welcome message size");
        }

        let mut welcome_nonce = [0u8; CRYPTO_BOX_NONCEBYTES];
        welcome_nonce[..8].copy_from_slice(b"WELCOME-");
        welcome_nonce[8..24].copy_from_slice(&msg_data[8..24]);

        let server_pk = box_::PublicKey(self.server_key);
        let cn_sk = box_::SecretKey(self.cn_secret);

        let welcome_box = &msg_data[24..168];
        let welcome_plaintext = box_::open(
            welcome_box,
            &box_::Nonce(welcome_nonce),
            &server_pk,
            &cn_sk,
        ).map_err(|_| "Failed to open welcome box")?;

        self.cn_server.copy_from_slice(&welcome_plaintext[..32]);
        self.cn_cookie.copy_from_slice(&welcome_plaintext[32..128]);

        // Precompute the shared secret
        let cn_server_pk = box_::PublicKey(self.cn_server);
        let precom = box_::precompute(&cn_server_pk, &box_::SecretKey(self.cn_secret));
        cn_precom.copy_from_slice(&precom.0);

        Ok(())
    }

    pub fn produce_initiate(
        &self,
        cn_nonce: u64,
        metadata_plaintext: &[u8],
    ) -> Result<Vec<u8>, &'static str> {
        let mut vouch_nonce = [0u8; CRYPTO_BOX_NONCEBYTES];
        vouch_nonce[..8].copy_from_slice(b"VOUCH---");
        sodiumoxide::randombytes::randombytes_into(&mut vouch_nonce[8..24]);

        let mut vouch_plaintext = vec![0u8; CRYPTO_BOX_ZEROBYTES + 64];
        vouch_plaintext[CRYPTO_BOX_ZEROBYTES..][..32].copy_from_slice(&self.cn_public);
        vouch_plaintext[CRYPTO_BOX_ZEROBYTES + 32..][..32].copy_from_slice(&self.server_key);

        let cn_server_pk = box_::PublicKey(self.cn_server);
        let secret_sk = box_::SecretKey(self.secret_key);
        
        let vouch_box = box_::seal(
            &vouch_plaintext,
            &box_::Nonce(vouch_nonce),
            &cn_server_pk,
            &secret_sk,
        );

        let mut initiate_nonce = [0u8; CRYPTO_BOX_NONCEBYTES];
        initiate_nonce[..16].copy_from_slice(b"CurveZMQINITIATE");
        initiate_nonce[16..24].copy_from_slice(&cn_nonce.to_le_bytes());

        let mut initiate_plaintext = vec![0u8; CRYPTO_BOX_ZEROBYTES + 128 + metadata_plaintext.len()];
        let offset = CRYPTO_BOX_ZEROBYTES;
        initiate_plaintext[offset..offset + 32].copy_from_slice(&self.public_key);
        initiate_plaintext[offset + 32..offset + 48].copy_from_slice(&vouch_nonce[8..24]);
        initiate_plaintext[offset + 48..offset + 128].copy_from_slice(&vouch_box[CRYPTO_BOX_BOXZEROBYTES..]);
        initiate_plaintext[offset + 128..].copy_from_slice(metadata_plaintext);

        let initiate_box = box_::seal(
            &initiate_plaintext,
            &box_::Nonce(initiate_nonce),
            &cn_server_pk,
            &box_::SecretKey(self.cn_secret),
        );

        let mut initiate = Vec::with_capacity(113 + 128 + CRYPTO_BOX_BOXZEROBYTES + metadata_plaintext.len());
        initiate.extend_from_slice(b"\x08INITIATE");
        initiate.extend_from_slice(&self.cn_cookie);
        initiate.extend_from_slice(&cn_nonce.to_le_bytes());
        initiate.extend_from_slice(&initiate_box[CRYPTO_BOX_BOXZEROBYTES..]);

        Ok(initiate)
    }

    pub fn is_handshake_command_welcome(msg_data: &[u8]) -> bool {
        msg_data.starts_with(b"\x07WELCOME")
    }

    pub fn is_handshake_command_ready(msg_data: &[u8]) -> bool {
        msg_data.starts_with(b"\x05READY")
    }

    pub fn is_handshake_command_error(msg_data: &[u8]) -> bool {
        msg_data.starts_with(b"\x05ERROR")
    }
}