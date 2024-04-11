use std::net::{ToSocketAddrs, UdpSocket};

use crate::error::Error;

/// Sends a buffer on UDP broadcast
pub fn send_packet<A: ToSocketAddrs>(
    bind_addr: A,
    broadcast_addr: A,
    buffer: &[u8],
) -> Result<usize, Error> {
    let socket = UdpSocket::bind(bind_addr)?;
    socket.set_broadcast(true)?;
    Ok(socket.send_to(buffer, broadcast_addr)?)
}
