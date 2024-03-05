use serde::Serialize;
use sqlx::{PgPool, postgres::PgPoolOptions, types::{ipnetwork::IpNetwork, mac_address::MacAddress}};
use tracing::{debug, info};
use utoipa::ToSchema;

#[derive(Serialize, Debug)]
pub struct Device {
    pub id: String,
    pub mac: MacAddress,
    pub broadcast_addr: String,
    pub ip: IpNetwork,
    pub times: Option<Vec<i64>>
}

#[derive(ToSchema)]
#[schema(as = Device)]
pub struct DeviceSchema {
    pub id: String,
    pub mac: String,
    pub broadcast_addr: String,
    pub ip: String,
    pub times: Option<Vec<i64>>
}

pub async fn init_db_pool(db_url: &str) -> PgPool {
    debug!("attempt to connect dbPool to '{}'", db_url);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await
        .unwrap();

    info!("dbPool successfully connected to '{}'", db_url);

    pool
}
