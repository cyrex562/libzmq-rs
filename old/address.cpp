/* SPDX-License-Identifier: MPL-2.0 */

#include "precompiled.hpp"
#include "macros.hpp"
#include "address.hpp"
#include "ctx.hpp"
#include "err.hpp"
#include "tcp_address.hpp"
#include "udp_address.hpp"
#include "ipc_address.hpp"
#include "tipc_address.hpp"
#include "ws_address.hpp"

#if defined ZMQ_HAVE_VMCI
#include "vmci_address.hpp"
#endif

#include <string>
#include <sstream>

zmq::address_t::address_t (const std::string &protocol_,
                           const std::string &address_,
                           ctx_t *parent_) :
    protocol (protocol_), address (address_), parent (parent_)
{
    resolved.dummy = NULL;
}

zmq::address_t::~address_t ()
{
    if (protocol == protocol_name::tcp) {
        LIBZMQ_DELETE (resolved.tcp_addr);
    } else if (protocol == protocol_name::udp) {
        LIBZMQ_DELETE (resolved.udp_addr);
    }
#ifdef ZMQ_HAVE_WS
    else if (protocol == protocol_name::ws) {
        LIBZMQ_DELETE (resolved.ws_addr);
    }
#endif

#ifdef ZMQ_HAVE_WSS
    else if (protocol == protocol_name::wss) {
        LIBZMQ_DELETE (resolved.ws_addr);
    }
#endif

#if defined ZMQ_HAVE_IPC
    else if (protocol == protocol_name::ipc) {
        LIBZMQ_DELETE (resolved.ipc_addr);
    }
#endif
#if defined ZMQ_HAVE_TIPC
    else if (protocol == protocol_name::tipc) {
        LIBZMQ_DELETE (resolved.tipc_addr);
    }
#endif
#if defined ZMQ_HAVE_VMCI
    else if (protocol == protocol_name::vmci) {
        LIBZMQ_DELETE (resolved.vmci_addr);
    }
#endif
}

int zmq::address_t::to_string (std::string &addr_) const
{
    if (protocol == protocol_name::tcp && resolved.tcp_addr)
        return resolved.tcp_addr->to_string (addr_);
    if (protocol == protocol_name::udp && resolved.udp_addr)
        return resolved.udp_addr->to_string (addr_);
#ifdef ZMQ_HAVE_WS
    if (protocol == protocol_name::ws && resolved.ws_addr)
        return resolved.ws_addr->to_string (addr_);
#endif
#ifdef ZMQ_HAVE_WSS
    if (protocol == protocol_name::wss && resolved.ws_addr)
        return resolved.ws_addr->to_string (addr_);
#endif
#if defined ZMQ_HAVE_IPC
    if (protocol == protocol_name::ipc && resolved.ipc_addr)
        return resolved.ipc_addr->to_string (addr_);
#endif
#if defined ZMQ_HAVE_TIPC
    if (protocol == protocol_name::tipc && resolved.tipc_addr)
        return resolved.tipc_addr->to_string (addr_);
#endif
#if defined ZMQ_HAVE_VMCI
    if (protocol == protocol_name::vmci && resolved.vmci_addr)
        return resolved.vmci_addr->to_string (addr_);
#endif

    if (!protocol.empty () && !address.empty ()) {
        std::stringstream s;
        s << protocol << "://" << address;
        addr_ = s.str ();
        return 0;
    }
    addr_.clear ();
    return -1;
}

zmq::zmq_socklen_t zmq::get_socket_address (fd_t fd_,
                                            socket_end_t socket_end_,
                                            sockaddr_storage *ss_)
{
    zmq_socklen_t sl = static_cast<zmq_socklen_t> (sizeof (*ss_));

    const int rc =
      socket_end_ == socket_end_local
        ? getsockname (fd_, reinterpret_cast<struct sockaddr *> (ss_), &sl)
        : getpeername (fd_, reinterpret_cast<struct sockaddr *> (ss_), &sl);

    return rc != 0 ? 0 : sl;
}

use std::ptr::null_mut;
use std::ffi::CString;

mod precompiled;
mod macros;
mod address;
mod ctx;
mod err;
mod tcp_address;
mod udp_address;
mod ipc_address;
mod tipc_address;
mod ws_address;

#[cfg(feature = "vmci")]
mod vmci_address;

struct Address {
    protocol: String,
    address: String,
    parent: *mut ctx::Ctx,
    resolved: ResolvedAddress,
}

enum ResolvedAddress {
    Dummy(*mut ()),
    Tcp(*mut tcp_address::TcpAddress),
    Udp(*mut udp_address::UdpAddress),
    // Add other address types as needed
}

impl Address {
    fn new(protocol: &str, address: &str, parent: *mut ctx::Ctx) -> Address {
        Address {
            protocol: protocol.to_string(),
            address: address.to_string(),
            parent,
            resolved: ResolvedAddress::Dummy(null_mut()),
        }
    }
}

impl Drop for Address {
    fn drop(&mut self) {
        match self.protocol.as_str() {
            "tcp" => {
                if let ResolvedAddress::Tcp(addr) = self.resolved {
                    unsafe { Box::from_raw(addr) };
                }
            }
            "udp" => {
                if let ResolvedAddress::Udp(addr) = self.resolved {
                    unsafe { Box::from_raw(addr) };
                }
            }
            _ => {}
        }
    }
}
