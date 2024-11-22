use std::collections::HashMap;
use std::sync::atomic::AtomicI32;

// Constants
const CURVE_KEYSIZE: usize = 32;
const CURVE_KEYSIZE_Z85: usize = 40;
const BIND_DEV_SZ: usize = 16;

// Types
type Uid = u32;
type Gid = u32;
type Pid = i32;

#[derive(Default)]
pub struct TcpAddressMask {
    // Implementation details omitted for brevity
}

pub struct Options {
    // High-water marks for message pipes
    send_high_water_mark: i32,
    recv_high_water_mark: i32,

    // I/O thread affinity
    affinity: u64,

    // Socket routing id
    routing_id_size: u8,
    routing_id: [u8; 256],

    // Maximum transfer rate [kb/s]. Default 100kb/s
    rate: i32,

    // Reliability time interval [ms]. Default 10 seconds
    recovery_ivl: i32,

    // Sets the time-to-live field in every multicast packet sent
    multicast_hops: i32,

    // Sets the maximum transport data unit size in every multicast packet sent
    multicast_max_trans_data_unit_szu: i32,

    // SO_SNDBUF and SO_RCVBUF to be passed to underlying transport sockets
    send_buf_opt: i32,
    recv_buf_opt: i32,

    // Type of service (containing DSCP and ECN socket options)
    type_of_svc: i32,

    // Protocol-defined priority
    priority: i32,

    // Socket type
    socket_type: i8,

    // Linger time, in milliseconds
    linger: AtomicI32,

    // Maximum interval in milliseconds beyond which userspace will timeout connect()
    connect_timeout: i32,

    // Maximum interval in milliseconds beyond which TCP will timeout retransmitted packets
    tcp_max_retrans_intvl: i32,

    // Disable reconnect under certain conditions
    reconnect_stop: i32,

    // Minimum interval between attempts to reconnect, in milliseconds
    pub reconnect_intvl: i32,

    // Maximum interval between attempts to reconnect, in milliseconds
    pub reconnect_intvl_max: i32,

    // Maximum backlog for pending connections
    backlog: i32,

    // Maximal size of message to handle
    max_msg_sz: i64,

    // The timeout for send/recv operations for this socket, in milliseconds
    recv_timeo: i32,
    send_timeo: i32,

    // If true, IPv6 is enabled (as well as IPv4)
    ipv6: bool,

    // If 1, connecting pipes are not attached immediately
    immediate: i32,

    // If 1, (X)SUB socket should filter the messages
    filter: bool,

    // If true, the subscription matching is reversed
    invert_matching: bool,

    // If true, the routing id message is forwarded to the socket
    recv_routing_id: bool,

    // If true, router socket accepts non-zmq tcp connections
    pub raw_socket: bool,
    raw_notify: bool,

    // Address of SOCKS proxy
    socks_proxy_address: String,
    socks_proxy_username: String,
    socks_proxy_password: String,

    // TCP keep-alive settings
    tcp_keepalive: i32,
    tcp_keepalive_cnt: i32,
    tcp_keepalive_idle: i32,
    tcp_keepalive_intvl: i32,

    // TCP accept() filters
    tcp_accept_filters: Vec<TcpAddressMask>,

    // IPC accept() filters
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    ipc_uid_accept_filters: HashSet<Uid>,
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    ipc_gid_accept_filters: HashSet<Gid>,
    #[cfg(target_os = "linux")]
    ipc_pid_accept_filters: HashSet<Pid>,

    // Security mechanism
    mechanism: i32,
    as_server: i32,
    zap_domain: String,

    // Security credentials for PLAIN mechanism
    pub plain_username: String,
    pub plain_password: String,

    // Security credentials for CURVE mechanism
    curve_public_key: [u8; CURVE_KEYSIZE],
    pub curve_secret_key: [u8; CURVE_KEYSIZE],
    curve_server_key: [u8; CURVE_KEYSIZE],

    // GSSAPI security configuration
    #[cfg(feature = "gssapi")]
    gss_principal: String,
    #[cfg(feature = "gssapi")]
    gss_service_principal: String,
    #[cfg(feature = "gssapi")]
    gss_principal_nt: i32,
    #[cfg(feature = "gssapi")]
    gss_service_principal_nt: i32,
    #[cfg(feature = "gssapi")]
    gss_plaintext: bool,

    // Socket ID
    socket_id: i32,

    // If true, socket conflates outgoing/incoming messages
    conflate: bool,

    // Connection handshake timeout
    handshake_intvl: i32,

    connected: bool,

    // Heartbeat configuration
    heartbeat_ttl: u16,
    heartbeat_intvl: i32,
    heartbeat_timeo: i32,

    // VMCI configuration
    #[cfg(feature = "vmci")]
    vmci_buffer_size: u64,
    #[cfg(feature = "vmci")]
    vmci_buffer_min_size: u64,
    #[cfg(feature = "vmci")]
    vmci_buffer_max_size: u64,
    #[cfg(feature = "vmci")]
    vmci_connect_timeout: i32,

    // File descriptor to use
    use_fd: i32,

    // Device to bind to
    bound_device: String,

    // ZAP configuration
    pub(crate) zap_enforce_domain: bool,

    // Performance options
    loopback_fastpath: bool,
    multicast_loop: bool,
    in_batch_size: i32,
    out_batch_size: i32,
    zero_copy: bool,

    // Router notifications
    router_notify: i32,

    // Application metadata
    app_metadata: HashMap<String, String>,

    // Monitor event version
    monitor_event_version: i32,

    // WSS configuration
    #[cfg(feature = "wss")]
    wss_key_pem: String,
    #[cfg(feature = "wss")]
    wss_cert_pem: String,
    #[cfg(feature = "wss")]
    wss_trust_pem: String,
    #[cfg(feature = "wss")]
    wss_hostname: String,
    #[cfg(feature = "wss")]
    wss_trust_system: bool,

    // Protocol messages
    hello_msg: Vec<u8>,
    pub(crate) can_send_hello_msg: bool,
    disconnect_msg: Vec<u8>,
    can_recv_disconnect_msg: bool,
    hiccup_msg: Vec<u8>,
    pub(crate) can_recv_hiccup_msg: bool,

    // NORM options
    #[cfg(feature = "norm")]
    norm_mode: i32,
    #[cfg(feature = "norm")]
    norm_unicast_nacks: bool,
    #[cfg(feature = "norm")]
    norm_buffer_size: i32,
    #[cfg(feature = "norm")]
    norm_segment_size: i32,
    #[cfg(feature = "norm")]
    norm_block_size: i32,
    #[cfg(feature = "norm")]
    norm_num_parity: i32,
    #[cfg(feature = "norm")]
    norm_num_autoparity: i32,
    #[cfg(feature = "norm")]
    norm_push_enable: bool,

    // Busy polling configuration
    busy_poll: i32,
}

impl Options {
    pub fn new() -> Self {
        Options {
            send_high_water_mark: 1000,
            recv_high_water_mark: 1000,
            affinity: 0,
            routing_id_size: 0,
            routing_id: [0; 256],
            rate: 100,
            recovery_ivl: 10000,
            multicast_hops: 1,
            multicast_max_trans_data_unit_szu: 1500,
            send_buf_opt: -1,
            recv_buf_opt: -1,
            type_of_svc: 0,
            priority: 0,
            socket_type: -1,
            linger: AtomicI32::new(-1),
            connect_timeout: 0,
            tcp_max_retrans_intvl: 0,
            reconnect_stop: 0,
            reconnect_intvl: 100,
            reconnect_intvl_max: 0,
            backlog: 100,
            max_msg_sz: -1,
            recv_timeo: -1,
            send_timeo: -1,
            ipv6: false,
            immediate: 0,
            filter: false,
            invert_matching: false,
            recv_routing_id: false,
            raw_socket: false,
            raw_notify: true,
            socks_proxy_address: String::new(),
            socks_proxy_username: String::new(),
            socks_proxy_password: String::new(),
            tcp_keepalive: -1,
            tcp_keepalive_cnt: -1,
            tcp_keepalive_idle: -1,
            tcp_keepalive_intvl: -1,
            tcp_accept_filters: Vec::new(),
            #[cfg(any(target_os = "linux", target_os = "macos"))]
            ipc_uid_accept_filters: HashSet::new(),
            #[cfg(any(target_os = "linux", target_os = "macos"))]
            ipc_gid_accept_filters: HashSet::new(),
            #[cfg(target_os = "linux")]
            ipc_pid_accept_filters: HashSet::new(),
            mechanism: 0, // ZMQ_NULL
            as_server: 0,
            zap_domain: String::new(),
            plain_username: String::new(),
            plain_password: String::new(),
            curve_public_key: [0; CURVE_KEYSIZE],
            curve_secret_key: [0; CURVE_KEYSIZE],
            curve_server_key: [0; CURVE_KEYSIZE],
            socket_id: 0,
            conflate: false,
            handshake_intvl: 30000,
            connected: false,
            heartbeat_ttl: 0,
            heartbeat_intvl: 0,
            heartbeat_timeo: -1,
            use_fd: -1,
            bound_device: String::new(),
            zap_enforce_domain: false,
            loopback_fastpath: false,
            multicast_loop: true,
            in_batch_size: 8192,
            out_batch_size: 8192,
            zero_copy: true,
            router_notify: 0,
            app_metadata: HashMap::new(),
            monitor_event_version: 1,
            hello_msg: Vec::new(),
            can_send_hello_msg: false,
            disconnect_msg: Vec::new(),
            can_recv_disconnect_msg: false,
            hiccup_msg: Vec::new(),
            can_recv_hiccup_msg: false,
            busy_poll: 0,
            // Initialize feature-gated fields
            ..Default::default()
        }
    }

    // Method implementations would go here
    // The original C++ methods would need to be converted to Rust
}

impl Default for Options {
    fn default() -> Self {
        Options::new()
    }
}

// Helper functions would go here
// Convert the C++ free functions to Rust free functions or implement as associated functions
