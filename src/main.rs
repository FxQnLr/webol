use crate::{
    config::Config, routes::{device, start, status}, services::ping::{BroadcastCommand, StatusMap}, storage::Device
};
use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};
use dashmap::DashMap;
use std::{env, sync::Arc};
use time::UtcOffset;
use tokio::sync::broadcast::{channel, Sender};
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt::{self, time::OffsetTime},
    prelude::*,
    EnvFilter,
};
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

mod auth;
mod config;
mod storage;
mod error;
mod routes;
mod services;
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
            start::Payload,
            start::Response,
            device::Payload,
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
#[allow(deprecated)]
async fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;

    let config = Config::load()?;

    let time_format =
        time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
    let time = OffsetTime::new(UtcOffset::from_hms(config.timeoffset, 0, 0)?, time_format);

    let file_appender = tracing_appender::rolling::daily("logs", "webol.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
        .with(fmt::layer().with_timer(time))
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    Device::setup()?;

    let version = env!("CARGO_PKG_VERSION");
    info!("start webol v{}", version);

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
