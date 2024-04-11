use std::net::{ToSocketAddrs, UdpSocket};

use tracing::trace;

use crate::error::Error;

/// Creates the magic packet from a mac address
///
/// # Panics
///
/// Panics if `mac_addr` is an invalid mac
pub fn create_buffer(mac_addr: &str) -> Result<Vec<u8>, Error> {
    let mut mac = Vec::new();
    let sp = mac_addr.split(':');
    for f in sp {
        mac.push(u8::from_str_radix(f, 16)?);
    }
    let mut buf = vec![255; 6];
    for _ in 0..16 {
        for i in &mac {
            buf.push(*i);
        }
    }
    Ok(buf)
}

/// Sends a buffer on UDP broadcast
pub fn send_packet<A: ToSocketAddrs>(
    bind_addr: A,
    broadcast_addr: A,
    buffer: &[u8],
) -> Result<usize, Error> {
    let socket = UdpSocket::bind(bind_addr)?;
    socket.set_broadcast(true)?;
    trace!(?buffer ,"start with");
    Ok(socket.send_to(buffer, broadcast_addr)?)
}
