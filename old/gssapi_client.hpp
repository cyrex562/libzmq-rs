/* SPDX-License-Identifier: MPL-2.0 */

#ifndef __ZMQ_GSSAPI_CLIENT_HPP_INCLUDED__
#define __ZMQ_GSSAPI_CLIENT_HPP_INCLUDED__

#ifdef HAVE_LIBGSSAPI_KRB5

#include "gssapi_mechanism_base.hpp"

namespace zmq
{
class msg_t;
class session_base_t;

class gssapi_client_t ZMQ_FINAL : public gssapi_mechanism_base_t
{
  public:
    gssapi_client_t (session_base_t *session_, const options_t &options_);
    ~gssapi_client_t () ZMQ_FINAL;

    // mechanism implementation
    int next_handshake_command (msg_t *msg_) ZMQ_FINAL;
    int process_handshake_command (msg_t *msg_) ZMQ_FINAL;
    int encode (msg_t *msg_) ZMQ_FINAL;
    int decode (msg_t *msg_) ZMQ_FINAL;
    status_t status () const ZMQ_FINAL;

  private:
    enum state_t
    {
        call_next_init,
        recv_next_token,
        send_ready,
        recv_ready,
        connected
    };

    //  Human-readable principal name of the service we are connecting to
    char *service_name;

    gss_OID service_name_type;

    //  Current FSM state
    state_t state;

    //  Points to either send_tok or recv_tok
    //  during context initialization
    gss_buffer_desc *token_ptr;

    //  The desired underlying mechanism
    gss_OID_set_desc mechs;

    //  True iff client considers the server authenticated
    bool security_context_established;

    int initialize_context ();
    int produce_next_token (msg_t *msg_);
    int process_next_token (msg_t *msg_);
};
}

#endif

#endif
