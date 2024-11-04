#[cfg(windows)]
use winapi::shared::winerror::*;
#[cfg(windows)]
use winapi::um::errhandlingapi::RaiseException;

#[derive(Debug)]
pub enum ZmqError {
    Fsm,
    NoCompatProto,
    Term,
    MThread,
    HostUnreach,
    #[cfg(windows)]
    NotSupported,
    #[cfg(windows)]
    ProtoNotSupported,
    #[cfg(windows)]
    NoBuffers,
    #[cfg(windows)]
    NetDown,
    #[cfg(windows)]
    AddrInUse,
    #[cfg(windows)]
    AddrNotAvail,
    #[cfg(windows)]
    ConnRefused,
    #[cfg(windows)]
    InProgress,
    SystemError(std::io::Error),
}

impl ZmqError {
    pub fn to_string(&self) -> &'static str {
        match self {
            ZmqError::Fsm => "Operation cannot be accomplished in current state",
            ZmqError::NoCompatProto => "The protocol is not compatible with the socket type",
            ZmqError::Term => "Context was terminated",
            ZmqError::MThread => "No thread available",
            ZmqError::HostUnreach => "Host unreachable",
            #[cfg(windows)]
            ZmqError::NotSupported => "Not supported",
            #[cfg(windows)]
            ZmqError::ProtoNotSupported => "Protocol not supported",
            #[cfg(windows)]
            ZmqError::NoBuffers => "No buffer space available",
            #[cfg(windows)]
            ZmqError::NetDown => "Network is down",
            #[cfg(windows)]
            ZmqError::AddrInUse => "Address in use",
            #[cfg(windows)]
            ZmqError::AddrNotAvail => "Address not available",
            #[cfg(windows)]
            ZmqError::ConnRefused => "Connection refused",
            #[cfg(windows)]
            ZmqError::InProgress => "Operation in progress",
            ZmqError::SystemError(e) => e.to_string().as_str(),
        }
    }
}

#[cfg(windows)]
pub fn wsa_error_to_errno(errcode: i32) -> std::io::Error {
    use std::io::ErrorKind;
    
    let kind = match errcode {
        WSAEINTR => ErrorKind::Interrupted,
        WSAEBADF => ErrorKind::InvalidInput,
        WSAEACCES => ErrorKind::PermissionDenied,
        WSAEFAULT => ErrorKind::InvalidInput,
        WSAEINVAL => ErrorKind::InvalidInput,
        WSAEMFILE => ErrorKind::Other,
        WSAEWOULDBLOCK => ErrorKind::WouldBlock,
        WSAEINPROGRESS | WSAEALREADY => ErrorKind::WouldBlock,
        WSAENOTSOCK => ErrorKind::NotConnected,
        WSAEMSGSIZE => ErrorKind::InvalidData,
        WSAEADDRINUSE => ErrorKind::AddrInUse,
        WSAEADDRNOTAVAIL => ErrorKind::AddrNotAvailable,
        WSAENETDOWN => ErrorKind::NotConnected,
        WSAENETUNREACH => ErrorKind::NotConnected,
        WSAECONNABORTED => ErrorKind::ConnectionAborted,
        WSAECONNRESET => ErrorKind::ConnectionReset,
        WSAENOBUFS => ErrorKind::OutOfMemory,
        WSAENOTCONN => ErrorKind::NotConnected,
        WSAETIMEDOUT => ErrorKind::TimedOut,
        WSAECONNREFUSED => ErrorKind::ConnectionRefused,
        WSAEHOSTUNREACH => ErrorKind::NotConnected,
        _ => ErrorKind::Other,
    };
    
    std::io::Error::new(kind, "Windows socket error")
}

#[macro_export]
macro_rules! zmq_assert {
    ($cond:expr) => {
        if !$cond {
            eprintln!("Assertion failed: {} ({}:{})", 
                stringify!($cond), 
                file!(), 
                line!()
            );
            std::process::abort();
        }
    };
}

#[macro_export]
macro_rules! errno_assert {
    ($cond:expr) => {
        if !$cond {
            let err = std::io::Error::last_os_error();
            eprintln!("{} ({}:{})", 
                err, 
                file!(), 
                line!()
            );
            std::process::abort();
        }
    };
}

pub fn print_backtrace() {
    use backtrace::Backtrace;
    let bt = Backtrace::new();
    eprintln!("{:?}", bt);
}

pub fn zmq_abort(msg: &str) -> ! {
    eprintln!("Fatal error: {}", msg);
    #[cfg(windows)]
    unsafe {
        RaiseException(0x40000015, 0, 1, &(msg.as_ptr() as usize));
    }
    #[cfg(not(windows))]
    {
        print_backtrace();
        std::process::abort();
    }
}
