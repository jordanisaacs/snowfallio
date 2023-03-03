//! Network related
//! Currently, TCP/UnixStream/UnixDatagram are implemented.

mod listener_config;
pub mod tcp;
pub mod udp;
pub mod unix;

pub use listener_config::ListenerConfig;
pub use tcp::{TcpListener, TcpStream};
pub use unix::{Pipe, UnixDatagram, UnixListener, UnixStream};

// Copied from mio.
pub(crate) fn new_socket(
    domain: libc::c_int,
    socket_type: libc::c_int,
) -> std::io::Result<libc::c_int> {
    let socket_type = socket_type | libc::SOCK_CLOEXEC;

    // Gives a warning for platforms without SOCK_NONBLOCK.
    #[allow(clippy::let_and_return)]
    let socket = crate::syscall!(socket(domain, socket_type, 0));

    socket
}
