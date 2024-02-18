use crate::config::Config;
use crate::db::init_db_pool;
use crate::routes::device;
use crate::routes::start::start;
use crate::routes::status::status;
use crate::services::ping::StatusMap;
use axum::routing::{get, put};
use axum::{routing::post, Router};
use dashmap::DashMap;
use services::ping::BroadcastCommand;
use sqlx::PgPool;
use tracing_subscriber::fmt::time::UtcTime;
use std::env;
use std::sync::Arc;
use tokio::sync::broadcast::{channel, Sender};
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt,
    prelude::*,
    EnvFilter,
};

mod auth;
mod config;
mod db;
mod error;
mod routes;
mod services;
mod wol;

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;
    

    let time_format =
        time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
    let loc = UtcTime::new(time_format);

    tracing_subscriber::registry()
        .with(fmt::layer().with_timer(loc))
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let version = env!("CARGO_PKG_VERSION");

    let config = Config::load()?;

    info!("start webol v{}", version);

    let db = init_db_pool(&config.database_url).await;
    sqlx::migrate!().run(&db).await.unwrap();

    let (tx, _) = channel(32);

    let ping_map: StatusMap = DashMap::new();

    let shared_state = Arc::new(AppState {
        db,
        config: config.clone(),
        ping_send: tx,
        ping_map,
    });

    let app = Router::new()
        .route("/start", post(start))
        .route("/device", get(device::get))
        .route("/device", put(device::put))
        .route("/device", post(device::post))
        .route("/status", get(status))
        .with_state(shared_state);

    let addr = config.serveraddr;
    info!("start server on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

pub struct AppState {
    db: PgPool,
    config: Config,
    ping_send: Sender<BroadcastCommand>,
    ping_map: StatusMap,
}
