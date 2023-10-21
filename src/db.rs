use std::env;

use serde::Serialize;
use sqlx::{PgPool, postgres::PgPoolOptions};
use tracing::{debug, info};

use crate::error::WebolError;

#[derive(Serialize)]
pub struct Device {
    pub id: String,
    pub mac: String,
    pub broadcast_addr: String
}

impl Device {
    async fn init(db: &PgPool) -> Result<(), WebolError> {
        sqlx::query!(r#"
            CREATE TABLE IF NOT EXISTS "devices"
            (
                "id"                TEXT PRIMARY KEY NOT NULL,
                "mac"               TEXT NOT NULL,
                "broadcast_addr"    TEXT NOT NULL
            );"#
        ).execute(db).await.map_err(|err| WebolError::Server(Box::new(err)))?;

        Ok(())
    }
}

pub async fn setup_db(db: &PgPool) -> Result<(), WebolError> {
    Device::init(db).await
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
