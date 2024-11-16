use std::net::TcpStream;
#[cfg(unix)]
use std::os::unix::io::{AsRawFd, RawFd};

use winapi::um::winsock2::closesocket;

use crate::{address::Address, io_thread::IoThread, options::Options, random::generate_random, raw_engine::RawEngine, session_base::SessionBase, socket_base::SocketBase, types::ZmqRawFd, zmtp_engine::ZmtpEngine};

// Constants
const RETIRED_FD: ZmqRawFd = u64::MAX;

// Type aliases
type Handle = Option<ZmqRawFd>;

// Timer ID enum
#[derive(Eq, PartialEq)]
enum TimerId {
    ReconnectTimer = 1,
}

// Base traits
trait IoObject {
    fn in_event(&mut self);
    fn out_event(&mut self);
    fn timer_event(&mut self, id: TimerId);
}

trait Own {
    fn process_plug(&mut self);
    fn process_term(&mut self, linger: i32);
}

// Main struct
pub struct StreamConnecterBase {
    addr: Box<Address>,
    s: ZmqRawFd,
    handle: Handle,
    endpoint: String,
    socket: Box<SocketBase>,
    delayed_start: bool,
    reconnect_timer_started: bool,
    current_reconnect_ivl: i32,
    session: Box<dyn SessionBase>,
    options: Options,
}

impl StreamConnecterBase {
    pub fn new(
        io_thread: &IoThread,
        session: Box<SessionBase>,
        options: Options,
        addr: Box<Address>,
        delayed_start: bool,
    ) -> Self {
        let endpoint = addr.to_string();
        let socket = session.get_socket();

        Self {
            addr,
            s: RETIRED_FD,
            handle: None,
            endpoint,
            socket,
            delayed_start,
            reconnect_timer_started: false,
            current_reconnect_ivl: -1,
            session,
            options,
        }
    }

    fn add_reconnect_timer(&mut self) {
        if self.options.reconnect_ivl > 0 {
            let interval = self.get_new_reconnect_ivl();
            self.add_timer(interval, TimerId::ReconnectTimer);
            self.socket.event_connect_retried(&self.endpoint, interval);
            self.reconnect_timer_started = true;
        }
    }

    fn get_new_reconnect_ivl(&mut self) -> i32 {
        if self.options.reconnect_ivl_max > 0 {
            let candidate_interval = if self.current_reconnect_ivl == -1 {
                self.options.reconnect_ivl
            } else if self.current_reconnect_ivl > i32::MAX / 2 {
                i32::MAX
            } else {
                self.current_reconnect_ivl * 2
            };

            self.current_reconnect_ivl = if candidate_interval > self.options.reconnect_ivl_max {
                self.options.reconnect_ivl_max
            } else {
                candidate_interval
            };
            self.current_reconnect_ivl
        } else {
            if self.current_reconnect_ivl == -1 {
                self.current_reconnect_ivl = self.options.reconnect_ivl;
            }
            let random_jitter = generate_random() % self.options.reconnect_ivl;
            if self.current_reconnect_ivl < i32::MAX - random_jitter {
                self.current_reconnect_ivl + random_jitter
            } else {
                i32::MAX
            }
        }
    }

    fn rm_handle(&mut self) {
        if let Some(handle) = self.handle {
            self.rm_fd(handle);
            self.handle = None;
        }
    }

    fn close(&mut self) {
        if self.s != RETIRED_FD {
            // Platform-specific socket closing
            #[cfg(windows)]
            {
                unsafe { closesocket(self.s).expect("Failed to close socket") };
            }
            #[cfg(not(windows))]
            {
                unsafe { libc::close(self.s) };
            }

            self.socket.event_closed(&self.endpoint, self.s);
            self.s = RETIRED_FD;
        }
    }

    fn create_engine(&mut self, fd: ZmqRawFd, local_address: &str) {
        let endpoint_pair = EndpointPair::new(local_address, &self.endpoint);

        let engine = if self.options.raw_socket {
            Box::new(RawEngine::new(fd, &self.options, endpoint_pair))
        } else {
            Box::new(ZmtpEngine::new(fd, &self.options, endpoint_pair))
        };

        self.session.attach(engine);
        self.terminate();
        self.socket.event_connected(&endpoint_pair, fd);
    }
}

impl IoObject for StreamConnecterBase {
    fn in_event(&mut self) {
        self.out_event();
    }

    fn out_event(&mut self) {
        // Implementation left to derived types
    }

    fn timer_event(&mut self, id: TimerId) {
        if id == TimerId::ReconnectTimer {
            self.reconnect_timer_started = false;
            self.start_connecting();
        }
    }
}

impl Own for StreamConnecterBase {
    fn process_plug(&mut self) {
        if self.delayed_start {
            self.add_reconnect_timer();
        } else {
            self.start_connecting();
        }
    }

    fn process_term(&mut self, linger: i32) {
        if self.reconnect_timer_started {
            self.cancel_timer(TimerId::ReconnectTimer);
            self.reconnect_timer_started = false;
        }

        if self.handle.is_some() {
            self.rm_handle();
        }

        if self.s != RETIRED_FD {
            self.close();
        }

        self.process_term_base(linger);
    }
}
