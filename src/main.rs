use std::env;
use std::sync::Arc;
use axum::{Router, routing::post};
use axum::routing::{get, put};
use sqlx::PgPool;
use time::util::local_offset;
use tokio::sync::broadcast::{channel, Sender};
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{EnvFilter, fmt::{self, time::LocalTime}, prelude::*};
use crate::config::SETTINGS;
use crate::db::init_db_pool;
use crate::routes::device::{get_device, post_device, put_device};
use crate::routes::start::start;
use crate::services::ping::ws_ping;

mod auth;
mod config;
mod routes;
mod wol;
mod db;
mod error;
mod services;

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
    sqlx::migrate!().run(&db).await.unwrap();

    let (tx, _) = channel(32);
    
    let shared_state = Arc::new(AppState { db, ping_send: tx });

    let app = Router::new()
        .route("/start", post(start))
        .route("/device", get(get_device))
        .route("/device", put(put_device))
        .route("/device", post(post_device))
        .route("/status", get(ws_ping))
        .with_state(shared_state);

    let addr = SETTINGS.get_string("serveraddr").unwrap_or("0.0.0.0:7229".to_string());
    info!("start server on {}", addr);
    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

pub struct AppState {
    db: PgPool,
    ping_send: Sender<String>,
}
