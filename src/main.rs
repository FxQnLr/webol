use axum::{Router, routing::post};
use time::util::local_offset;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{EnvFilter, fmt::{self, time::LocalTime}, prelude::*};
use crate::routes::start::start;

mod auth;
mod routes;

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

    info!("Starting webol v{}", version);

    // build our application with a single route
    let app = Router::new()
        .route("/start", post(start));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

