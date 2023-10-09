use serde::Serialize;

#[derive(Serialize)]
pub struct Device {
    pub id: String,
    pub mac: String,
    pub broadcast_addr: String
}