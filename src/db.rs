use std::env;

use serde::Serialize;
use sqlx::{PgPool, postgres::PgPoolOptions};
use tracing::{debug, info};

#[derive(Serialize)]
pub struct Device {
    pub id: String,
    pub mac: String,
    pub broadcast_addr: String
}

pub async fn init_db_pool() -> PgPool {
    #[cfg(not(debug_assertions))]
    let db_url = SETTINGS.get_string("database.url").unwrap();

    #[cfg(debug_assertions)]
    let db_url = env::var("DATABASE_URL").unwrap();

    debug!("attempt to connect dbPool to '{}'", db_url);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap();

    info!("dbPool successfully connected to '{}'", db_url);

    pool
}
