use std::net::{SocketAddr, UdpSocket};
use std::num::ParseIntError;

/// Creates the magic packet from a mac address
///
/// # Panics
///
/// Panics if `mac_addr` is an invalid mac
pub fn create_buffer(mac_addr: &str) -> Result<Vec<u8>, ParseIntError> {
    let mut mac = Vec::new();
    let sp = mac_addr.split(':');
    for f in sp {
        mac.push(u8::from_str_radix(f, 16)?);
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
pub fn send_packet(bind_addr: &SocketAddr, broadcast_addr: &SocketAddr, buffer: Vec<u8>) -> Result<usize, std::io::Error> {
    let socket = UdpSocket::bind(bind_addr)?;
    socket.set_broadcast(true)?;
    socket.send_to(&buffer, broadcast_addr)
}