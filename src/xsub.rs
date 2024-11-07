use std::ffi::c_void;
use std::os::raw::c_int;

#[cfg(feature = "use_radix_tree")]
use crate::radix_tree::RadixTree as SubscriptionStore;
#[cfg(not(feature = "use_radix_tree"))]
use crate::trie::TrieWithSize as SubscriptionStore;

use crate::context::Context;
use crate::dist::Dist;
use crate::fair_queue::FairQueue;
use crate::message::Message;
use crate::pipe::Pipe;
use crate::socket_base::SocketBase;

pub struct XSub {
    socket_base: SocketBase,
    fq: FairQueue,
    dist: Dist,
    subscriptions: SubscriptionStore,
    verbose_unsubs: bool,
    has_message: bool,
    message: Message,
    more_send: bool,
    more_recv: bool,
    process_subscribe: bool,
    only_first_subscribe: bool,
}

impl XSub {
    pub fn new(parent: &mut Context, tid: u32, sid: i32) -> Self {
        let mut socket = SocketBase::new(parent, tid, sid);
        socket.set_type(ZMQ_XSUB);
        socket.set_linger(0);

        Self {
            socket_base: socket,
            fq: FairQueue::new(),
            dist: Dist::new(),
            subscriptions: SubscriptionStore::new(),
            verbose_unsubs: false,
            has_message: false,
            message: Message::new(),
            more_send: false,
            more_recv: false,
            process_subscribe: false,
            only_first_subscribe: false,
        }
    }

    pub fn attach_pipe(&mut self, pipe: &mut Pipe, _subscribe_to_all: bool, _locally_initiated: bool) {
        self.fq.attach(pipe);
        self.dist.attach(pipe);

        // Send all cached subscriptions to new upstream peer
        self.subscriptions.apply(Self::send_subscription, pipe);
        pipe.flush();
    }

    pub fn set_sockopt(&mut self, option: c_int, optval: *const c_void, optvallen: usize) -> c_int {
        match option {
            ZMQ_ONLY_FIRST_SUBSCRIBE => {
                if optvallen != std::mem::size_of::<c_int>() {
                    errno!(EINVAL);
                    return -1;
                }
                let val = unsafe { *(optval as *const c_int) };
                if val < 0 {
                    errno!(EINVAL);
                    return -1;
                }
                self.only_first_subscribe = val != 0;
                0
            }
            #[cfg(feature = "draft")]
            ZMQ_XSUB_VERBOSE_UNSUBSCRIBE => {
                self.verbose_unsubs = unsafe { *(optval as *const c_int) } != 0;
                0
            }
            _ => {
                errno!(EINVAL);
                -1
            }
        }
    }

    fn send_subscription(data: &[u8], pipe: &mut Pipe) {
        let mut msg = Message::new();
        msg.init_subscribe(data);
        
        if !pipe.write(&msg) {
            msg.close();
        }
    }

    // ... Additional method implementations omitted for brevity
    // Convert other methods like xrecv, xsend, match, etc following 
    // similar patterns of Rust idioms and memory safety
}
