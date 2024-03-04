use crate::{
    config::Config,
    db::init_db_pool,
    routes::{device, start, status},
    services::ping::{BroadcastCommand, StatusMap},
};
use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};
use dashmap::DashMap;
use sqlx::PgPool;
use std::{env, sync::Arc};
use tokio::sync::broadcast::{channel, Sender};
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt::{self, time::UtcTime},
    prelude::*,
    EnvFilter,
};
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

mod config;
mod db;
mod error;
mod extractors;
mod routes;
mod services;
mod wol;

#[derive(OpenApi)]
#[openapi(
    paths(
        start::start,
        device::get,
        device::get_path,
        device::post,
        device::put,
    ),
    components(
        schemas(
            start::Payload,
            start::Response,
            device::PutDevicePayload,
            device::GetDevicePayload,
            device::PostDevicePayload,
            db::DeviceSchema,
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
        .route("/start", post(start::start))
        .route(
            "/device",
            post(device::post).get(device::get).put(device::put),
        )
        .route("/device/:id", get(device::get_path))
        .route("/status", get(status::status))
        .route_layer(from_fn_with_state(shared_state.clone(), extractors::auth))
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
    db: PgPool,
    config: Config,
    ping_send: Sender<BroadcastCommand>,
    ping_map: StatusMap,
}
