use std::env;
use std::sync::Arc;
use axum::{Router, routing::post};
use axum::routing::{get, put};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use time::util::local_offset;
use tracing::{debug, info, level_filters::LevelFilter};
use tracing_subscriber::{EnvFilter, fmt::{self, time::LocalTime}, prelude::*};
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

    info!("starting webol v{}", version);

    let db = init_db_pool().await;

    let shared_state = Arc::new(AppState { db });

    // build our application with a single route
    let app = Router::new()
        .route("/start", post(start))
        .route("/device", get(get_device))
        .route("/device", put(put_device))
        .route("/device", post(post_device))
        .with_state(shared_state);

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

pub struct AppState {
    db: PgPool
}

async fn init_db_pool() -> PgPool {
    let db_url = env::var("DATABASE_URL").unwrap();

    debug!("attempting to connect dbPool to '{}'", db_url);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap();

    info!("dbPool successfully connected to '{}'", db_url);

    pool
}