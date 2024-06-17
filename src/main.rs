use crate::{
    config::Config,
    routes::{device, start, status},
    services::ping::{BroadcastCommand, StatusMap},
    storage::Device,
};
use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};
use dashmap::DashMap;
use std::{env, sync::Arc};
use tokio::sync::broadcast::{channel, Sender};
use tracing::{info, level_filters::LevelFilter, trace};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

mod auth;
mod config;
mod error;
mod routes;
mod services;
mod storage;
mod wol;

#[derive(OpenApi)]
#[openapi(
    paths(
        start::post,
        start::get,
        device::get,
        device::post,
        device::put,
    ),
    components(
        schemas(
            start::SPayload,
            start::Response,
            device::DPayload,
            storage::DeviceSchema,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Webol", description = "Webol API")
    )
)]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("Authorization"))),
            );
        }
    }
}

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;

    let config = Config::load()?;

    let writer_time =
        tracing_subscriber::fmt::time::ChronoLocal::new("%Y-%m-%d %H:%M:%S%.6f%:z".to_string());
    let time = tracing_subscriber::fmt::time::ChronoLocal::new("%Y-%m-%d %H:%M:%S%:z".to_string());

    let file_appender = tracing_appender::rolling::daily("logs", "webol.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_timer(writer_time)
                .with_writer(non_blocking)
                .with_ansi(false),
        )
        .with(fmt::layer().with_timer(time))
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();
    trace!("logging initialized");

    Device::setup()?;

    let version = env!("CARGO_PKG_VERSION");
    info!(?version, "start webol");

    let (tx, _) = channel(32);

    let ping_map: StatusMap = DashMap::new();

    let shared_state = AppState {
        config: config.clone(),
        ping_send: tx,
        ping_map,
    };

    let app = Router::new()
        .route("/start/:id", post(start::post).get(start::get))
        .route("/device", post(device::post).put(device::put))
        .route("/device/:id", get(device::get))
        .route("/status", get(status::status))
        .route_layer(from_fn_with_state(shared_state.clone(), auth::auth))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(Arc::new(shared_state));

    let addr = config.serveraddr;
    info!("start server on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
pub struct AppState {
    config: Config,
    ping_send: Sender<BroadcastCommand>,
    ping_map: StatusMap,
}
