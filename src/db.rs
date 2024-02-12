use serde::Serialize;
use sqlx::{PgPool, postgres::PgPoolOptions};
use tracing::{debug, info};

#[derive(Serialize, Debug)]
pub struct Device {
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
