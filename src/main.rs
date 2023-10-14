use std::env;
use std::sync::Arc;
use axum::{Router, routing::post};
use axum::routing::{get, put};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use time::util::local_offset;
use tracing::{debug, info, level_filters::LevelFilter};
use tracing_subscriber::{EnvFilter, fmt::{self, time::LocalTime}, prelude::*};
use crate::config::SETTINGS;
use crate::routes::device::{get_device, post_device, put_device};
use crate::routes::start::start;

mod auth;
mod config;
mod routes;
mod wol;
mod db;
mod error;

#[tokio::main]
async fn main() {
    unsafe { local_offset::set_soundness(local_offset::Soundness::Unsound); }
    let time_format =
        time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
    let loc = LocalTime::new(time_format);

    tracing_subscriber::registry()
        .with(fmt::layer()
            .with_timer(loc)
        )
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let version = env!("CARGO_PKG_VERSION");

    info!("start webol v{}", version);

    let db = init_db_pool().await;

    let shared_state = Arc::new(AppState { db });

    let app = Router::new()
        .route("/start", post(start))
        .route("/device", get(get_device))
        .route("/device", put(put_device))
        .route("/device", post(post_device))
        .with_state(shared_state);

    let addr = SETTINGS.get_string("serveraddr").unwrap_or("0.0.0.0:7229".to_string());
    info!("start server on {}", addr);
    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

pub struct AppState {
    db: PgPool
}

async fn init_db_pool() -> PgPool {
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
