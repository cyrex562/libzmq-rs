use std::collections::HashMap;

// Socket type constants
const SOCKET_TYPE_PAIR: &str = "PAIR";
const SOCKET_TYPE_PUB: &str = "PUB";
const SOCKET_TYPE_SUB: &str = "SUB";
const SOCKET_TYPE_REQ: &str = "REQ";
const SOCKET_TYPE_REP: &str = "REP";
const SOCKET_TYPE_DEALER: &str = "DEALER";
const SOCKET_TYPE_ROUTER: &str = "ROUTER";
const SOCKET_TYPE_PULL: &str = "PULL";
const SOCKET_TYPE_PUSH: &str = "PUSH";
const SOCKET_TYPE_XPUB: &str = "XPUB";
const SOCKET_TYPE_XSUB: &str = "XSUB";
const SOCKET_TYPE_STREAM: &str = "STREAM";

#[cfg(feature = "draft")]
const SOCKET_TYPE_SERVER: &str = "SERVER";
#[cfg(feature = "draft")]
const SOCKET_TYPE_CLIENT: &str = "CLIENT";
#[cfg(feature = "draft")]
const SOCKET_TYPE_RADIO: &str = "RADIO";
#[cfg(feature = "draft")]
const SOCKET_TYPE_DISH: &str = "DISH";
#[cfg(feature = "draft")]
const SOCKET_TYPE_GATHER: &str = "GATHER";
#[cfg(feature = "draft")]
const SOCKET_TYPE_SCATTER: &str = "SCATTER";
#[cfg(feature = "draft")]
const SOCKET_TYPE_DGRAM: &str = "DGRAM";
#[cfg(feature = "draft")]
const SOCKET_TYPE_PEER: &str = "PEER";
#[cfg(feature = "draft")]
const SOCKET_TYPE_CHANNEL: &str = "CHANNEL";

const ZMTP_PROPERTY_SOCKET_TYPE: &str = "Socket-Type";
const ZMTP_PROPERTY_IDENTITY: &str = "Identity";

#[derive(Debug, PartialEq)]
pub enum Status {
    Handshaking,
    Ready,
    Error,
}

#[derive(Clone)]
pub struct Options {
    socket_type: i32,
    routing_id: Vec<u8>,
    routing_id_size: usize,
    recv_routing_id: bool,
    app_metadata: HashMap<String, String>,
}

pub struct Blob {
    data: Vec<u8>,
}

impl Blob {
    fn new() -> Self {
        Self { data: Vec::new() }
    }

    fn set(&mut self, data: &[u8]) {
        self.data = data.to_vec();
    }

    fn size(&self) -> usize {
        self.data.len()
    }

    fn data(&self) -> &[u8] {
        &self.data
    }
}

pub struct Mechanism {
    options: Options,
    zmtp_properties: HashMap<String, String>,
    zap_properties: HashMap<String, String>,
    routing_id: Blob,
    user_id: Blob,
}

impl Mechanism {
    pub fn new(options: Options) -> Self {
        Self {
            options,
            zmtp_properties: HashMap::new(),
            zap_properties: HashMap::new(),
            routing_id: Blob::new(),
            user_id: Blob::new(),
        }
    }

    pub fn set_peer_routing_id(&mut self, id: &[u8]) {
        self.routing_id.set(id);
    }

    pub fn set_user_id(&mut self, user_id: &[u8]) {
        self.user_id.set(user_id);
        self.zap_properties.insert(
            String::from("User-Id"),
            String::from_utf8_lossy(user_id).into_owned(),
        );
    }

    pub fn get_user_id(&self) -> &Blob {
        &self.user_id
    }

    fn socket_type_string(socket_type: i32) -> &'static str {
        match socket_type {
            0 => SOCKET_TYPE_PAIR,
            1 => SOCKET_TYPE_PUB,
            2 => SOCKET_TYPE_SUB,
            3 => SOCKET_TYPE_REQ,
            4 => SOCKET_TYPE_REP,
            5 => SOCKET_TYPE_DEALER,
            6 => SOCKET_TYPE_ROUTER,
            7 => SOCKET_TYPE_PULL,
            8 => SOCKET_TYPE_PUSH,
            9 => SOCKET_TYPE_XPUB,
            10 => SOCKET_TYPE_XSUB,
            11 => SOCKET_TYPE_STREAM,
            #[cfg(feature = "draft")]
            12 => SOCKET_TYPE_SERVER,
            #[cfg(feature = "draft")]
            13 => SOCKET_TYPE_CLIENT,
            #[cfg(feature = "draft")]
            14 => SOCKET_TYPE_RADIO,
            #[cfg(feature = "draft")]
            15 => SOCKET_TYPE_DISH,
            #[cfg(feature = "draft")]
            16 => SOCKET_TYPE_GATHER,
            #[cfg(feature = "draft")]
            17 => SOCKET_TYPE_SCATTER,
            #[cfg(feature = "draft")]
            18 => SOCKET_TYPE_DGRAM,
            #[cfg(feature = "draft")]
            19 => SOCKET_TYPE_PEER,
            #[cfg(feature = "draft")]
            20 => SOCKET_TYPE_CHANNEL,
            _ => panic!("Invalid socket type"),
        }
    }

    fn check_socket_type(&self, peer_type: &str) -> bool {
        match self.options.socket_type {
            3 /* REQ */ => peer_type == SOCKET_TYPE_REP || peer_type == SOCKET_TYPE_ROUTER,
            4 /* REP */ => peer_type == SOCKET_TYPE_REQ || peer_type == SOCKET_TYPE_DEALER,
            5 /* DEALER */ => {
                peer_type == SOCKET_TYPE_REP
                    || peer_type == SOCKET_TYPE_DEALER
                    || peer_type == SOCKET_TYPE_ROUTER
            }
            6 /* ROUTER */ => {
                peer_type == SOCKET_TYPE_REQ
                    || peer_type == SOCKET_TYPE_DEALER
                    || peer_type == SOCKET_TYPE_ROUTER
            }
            8 /* PUSH */ => peer_type == SOCKET_TYPE_PULL,
            7 /* PULL */ => peer_type == SOCKET_TYPE_PUSH,
            1 /* PUB */ => peer_type == SOCKET_TYPE_SUB || peer_type == SOCKET_TYPE_XSUB,
            2 /* SUB */ => peer_type == SOCKET_TYPE_PUB || peer_type == SOCKET_TYPE_XPUB,
            9 /* XPUB */ => peer_type == SOCKET_TYPE_SUB || peer_type == SOCKET_TYPE_XSUB,
            10 /* XSUB */ => peer_type == SOCKET_TYPE_PUB || peer_type == SOCKET_TYPE_XPUB,
            0 /* PAIR */ => peer_type == SOCKET_TYPE_PAIR,
            #[cfg(feature = "draft")]
            12 /* SERVER */ => peer_type == SOCKET_TYPE_CLIENT,
            #[cfg(feature = "draft")]
            13 /* CLIENT */ => peer_type == SOCKET_TYPE_SERVER,
            #[cfg(feature = "draft")]
            14 /* RADIO */ => peer_type == SOCKET_TYPE_DISH,
            #[cfg(feature = "draft")]
            15 /* DISH */ => peer_type == SOCKET_TYPE_RADIO,
            #[cfg(feature = "draft")]
            16 /* GATHER */ => peer_type == SOCKET_TYPE_SCATTER,
            #[cfg(feature = "draft")]
            17 /* SCATTER */ => peer_type == SOCKET_TYPE_GATHER,
            #[cfg(feature = "draft")]
            18 /* DGRAM */ => peer_type == SOCKET_TYPE_DGRAM,
            #[cfg(feature = "draft")]
            19 /* PEER */ => peer_type == SOCKET_TYPE_PEER,
            #[cfg(feature = "draft")]
            20 /* CHANNEL */ => peer_type == SOCKET_TYPE_CHANNEL,
            _ => false,
        }
    }
}

// Trait for mechanism implementations
pub trait MechanismOps {
    fn next_handshake_command(&mut self) -> Vec<u8>;
    fn process_handshake_command(&mut self, msg: &[u8]) -> i32;
    fn encode(&mut self, msg: &mut Vec<u8>) -> i32 {
        0
    }
    fn decode(&mut self, msg: &mut Vec<u8>) -> i32 {
        0
    }
    fn zap_msg_available(&mut self) -> i32 {
        0
    }
    fn status(&self) -> Status;
}
