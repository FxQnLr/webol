use std::sync::Arc;
use axum::{Router, routing::post};
use sqlx::SqlitePool;
use time::util::local_offset;
use tracing::{debug, info, level_filters::LevelFilter};
use tracing_subscriber::{EnvFilter, fmt::{self, time::LocalTime}, prelude::*};
use crate::routes::start::start;

mod auth;
mod config;
mod routes;
mod wol;
mod db;

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

    debug!("connecting to db");
    let db = SqlitePool::connect("sqlite:devices.sqlite").await.unwrap();
    sqlx::migrate!().run(&db).await.unwrap();
    info!("connected to db");

    let version = env!("CARGO_PKG_VERSION");

    info!("starting webol v{}", version);

    let shared_state = Arc::new(AppState { db });

    // build our application with a single route
    let app = Router::new()
        .route("/start", post(start))
        .with_state(shared_state);

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

pub struct AppState {
    db: SqlitePool
}