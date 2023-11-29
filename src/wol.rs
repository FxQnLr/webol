use std::net::{SocketAddr, UdpSocket};

use crate::error::WebolError;

/// Creates the magic packet from a mac address
///
/// # Panics
///
/// Panics if `mac_addr` is an invalid mac
pub fn create_buffer(mac_addr: &str) -> Result<Vec<u8>, WebolError> {
    let mut mac = Vec::new();
    let sp = mac_addr.split(':');
    for f in sp {
        mac.push(u8::from_str_radix(f, 16).map_err(WebolError::BufferParse)?);
    };
    let mut buf = vec![255; 6];
    for _ in 0..16 {
        for i in &mac {
            buf.push(*i);
        }
    }
    Ok(buf)
}

/// Sends a buffer on UDP broadcast
pub fn send_packet(bind_addr: &SocketAddr, broadcast_addr: &SocketAddr, buffer: Vec<u8>) -> Result<usize, WebolError> {
    let socket = UdpSocket::bind(bind_addr).map_err(WebolError::Broadcast)?;
    socket.set_broadcast(true).map_err(WebolError::Broadcast)?;
    socket.send_to(&buffer, broadcast_addr).map_err(WebolError::Broadcast)
}
