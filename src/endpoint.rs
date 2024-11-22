#[derive(Clone, Copy, PartialEq)]
pub enum EndpointType {
    None,    // a connection-less endpoint
    Bind,    // a connection-oriented bind endpoint
    Connect, // a connection-oriented connect endpoint
}

pub struct EndpointUriPair {
    local: String,
    remote: String,
    local_type: EndpointType,
}

impl EndpointUriPair {
    pub fn new() -> Self {
        Self {
            local: String::new(),
            remote: String::new(),
            local_type: EndpointType::None,
        }
    }

    pub fn with_values(local: &str, remote: &str, local_type: EndpointType) -> Self {
        Self {
            local: local.to_string(),
            remote: remote.to_string(),
            local_type,
        }
    }

    pub fn identifier(&self) -> &String {
        match self.local_type {
            EndpointType::Bind => &self.local,
            _ => &self.remote,
        }
    }

    pub fn clash(&self) -> bool {
        self.local == self.remote
    }
}

pub fn make_unconnected_connect_endpoint_pair(endpoint: &str) -> EndpointUriPair {
    EndpointUriPair::with_values("", endpoint, EndpointType::Connect)
}

pub fn make_unconnected_bind_endpoint_pair(endpoint: &str) -> EndpointUriPair {
    EndpointUriPair::with_values(endpoint, "", EndpointType::Bind)
}
