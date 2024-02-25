use crate::config::Config;
use crate::db::init_db_pool;
use crate::routes::device;
use crate::routes::start::start;
use crate::routes::status::status;
use crate::services::ping::StatusMap;
use axum::middleware::from_fn_with_state;
use axum::routing::{get, put};
use axum::{routing::post, Router};
use dashmap::DashMap;
use services::ping::BroadcastCommand;
use sqlx::PgPool;
use std::env;
use std::sync::Arc;
use tokio::sync::broadcast::{channel, Sender};
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod config;
mod db;
mod error;
mod extractors;
mod routes;
mod services;
mod wol;

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;

    let time_format =
        time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
    let loc = UtcTime::new(time_format);

    let file_appender = tracing_appender::rolling::daily("logs", "webol.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
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

    let shared_state = AppState {
        db,
        config: config.clone(),
        ping_send: tx,
        ping_map,
    };

    let app = Router::new()
        .route("/start", post(start))
        .route("/device", get(device::get))
        .route("/device", put(device::put))
        .route("/device", post(device::post))
        .route("/status", get(status))
        .route_layer(from_fn_with_state(shared_state.clone(), extractors::auth))
        .with_state(Arc::new(shared_state));

    let addr = config.serveraddr;
    info!("start server on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
pub struct AppState {
    db: PgPool,
    config: Config,
    ping_send: Sender<BroadcastCommand>,
    ping_map: StatusMap,
}
