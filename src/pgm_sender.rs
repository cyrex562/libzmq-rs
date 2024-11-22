#[cfg(feature = "openpgm")]
use {
    crate::endpoint_uri_pair::EndpointUriPair, crate::io_object::IoObject,
    crate::io_thread::IoThread, crate::message::Message, crate::pgm_socket::PgmSocket,
    crate::session_base::SessionBase, crate::v1_encoder::V1Encoder, std::ptr,
};

// Timer IDs
const TX_TIMER_ID: i32 = 0xa0;
const RX_TIMER_ID: i32 = 0xa1;

#[cfg(feature = "openpgm")]
pub struct PgmSender {
    io_object: IoObject,
    has_tx_timer: bool,
    has_rx_timer: bool,
    session: Option<Box<SessionBase>>,
    encoder: V1Encoder,
    more_flag: bool,
    pgm_socket: PgmSocket,
    handle: Handle,
    uplink_handle: Handle,
    rdata_notify_handle: Handle,
    pending_notify_handle: Handle,
    out_buffer: Option<Vec<u8>>,
    out_buffer_size: usize,
    write_size: usize,
}

#[cfg(feature = "openpgm")]
impl PgmSender {
    pub fn new(parent: &IoThread, options: &Options) -> Self {
        Self {
            io_object: IoObject::new(parent),
            has_tx_timer: false,
            has_rx_timer: false,
            session: None,
            encoder: V1Encoder::new(0),
            more_flag: false,
            pgm_socket: PgmSocket::new(false, options),
            handle: Handle::null(),
            uplink_handle: Handle::null(),
            rdata_notify_handle: Handle::null(),
            pending_notify_handle: Handle::null(),
            out_buffer: None,
            out_buffer_size: 0,
            write_size: 0,
        }
    }

    pub fn init(&mut self, udp_encapsulation: bool, network: &str) -> Result<(), Error> {
        self.pgm_socket.init(udp_encapsulation, network)?;

        self.out_buffer_size = self.pgm_socket.get_max_tsdu_size();
        self.out_buffer = Some(vec![0; self.out_buffer_size]);

        Ok(())
    }

    pub fn plug(&mut self, _io_thread: &IoThread, session: Box<SessionBase>) {
        self.session = Some(session);

        let (downlink_fd, uplink_fd, rdata_fd, pending_fd) = self.pgm_socket.get_sender_fds();

        self.handle = self.add_fd(downlink_fd);
        self.uplink_handle = self.add_fd(uplink_fd);
        self.rdata_notify_handle = self.add_fd(rdata_fd);
        self.pending_notify_handle = self.add_fd(pending_fd);

        self.set_pollin(self.uplink_handle);
        self.set_pollin(self.rdata_notify_handle);
        self.set_pollin(self.pending_notify_handle);
        self.set_pollout(self.handle);
    }

    fn unplug(&mut self) {
        if self.has_rx_timer {
            self.cancel_timer(RX_TIMER_ID);
            self.has_rx_timer = false;
        }

        if self.has_tx_timer {
            self.cancel_timer(TX_TIMER_ID);
            self.has_tx_timer = false;
        }

        self.rm_fd(self.handle);
        self.rm_fd(self.uplink_handle);
        self.rm_fd(self.rdata_notify_handle);
        self.rm_fd(self.pending_notify_handle);
        self.session = None;
    }

    pub fn in_event(&mut self) {
        if self.has_rx_timer {
            self.cancel_timer(RX_TIMER_ID);
            self.has_rx_timer = false;
        }

        match self.pgm_socket.process_upstream() {
            Err(e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::OutOfMemory => {
                let timeout = self.pgm_socket.get_rx_timeout();
                self.add_timer(timeout, RX_TIMER_ID);
                self.has_rx_timer = true;
            }
            _ => {}
        }
    }

    pub fn out_event(&mut self) {
        if self.write_size == 0 {
            let buffer = self.out_buffer.as_mut().unwrap();
            let mut offset = 0xffffu16;
            let mut bytes = 0;

            while bytes < self.out_buffer_size - 2 {
                if !self.more_flag && offset == 0xffff {
                    offset = bytes as u16;
                }

                if let Some(session) = &mut self.session {
                    match session.pull_msg() {
                        Ok(msg) => {
                            self.more_flag = msg.has_more();
                            self.encoder.load_msg(msg);
                            bytes += self.encoder.encode(&mut buffer[2 + bytes..]);
                        }
                        Err(_) => break,
                    }
                }
            }

            if bytes == 0 {
                self.reset_pollout(self.handle);
                return;
            }

            buffer[0] = (offset >> 8) as u8;
            buffer[1] = offset as u8;
            self.write_size = bytes + 2;
        }

        if self.has_tx_timer {
            self.cancel_timer(TX_TIMER_ID);
            self.set_pollout(self.handle);
            self.has_tx_timer = false;
        }

        match self
            .pgm_socket
            .send(&self.out_buffer.as_ref().unwrap()[..self.write_size])
        {
            Ok(n) if n == self.write_size => {
                self.write_size = 0;
            }
            Err(e) if e.kind() == ErrorKind::OutOfMemory => {
                let timeout = self.pgm_socket.get_tx_timeout();
                self.add_timer(timeout, TX_TIMER_ID);
                self.reset_pollout(self.handle);
                self.has_tx_timer = true;
            }
            _ => {}
        }
    }

    pub fn timer_event(&mut self, token: i32) {
        match token {
            RX_TIMER_ID => {
                self.has_rx_timer = false;
                self.in_event();
            }
            TX_TIMER_ID => {
                self.has_tx_timer = false;
                self.set_pollout(self.handle);
                self.out_event();
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(feature = "openpgm")]
impl Engine for PgmSender {
    fn has_handshake_stage(&self) -> bool {
        false
    }

    fn get_endpoint(&self) -> &EndpointUriPair {
        &EndpointUriPair::empty()
    }
}
