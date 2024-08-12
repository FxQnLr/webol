use crate::error::Error;
use crate::storage::Device;
use axum::Json;
use serde_json::{json, Value};
use tracing::{debug, info};

#[utoipa::path(
    get,
    path = "/devices",
    responses(
        (status = 200, description = "Get an array of all `Device`s", body = [Vec<Device>])
    ),
    security((), ("api_key" = []))
)]
pub async fn get(
) -> Result<Json<Value>, Error> {
    info!("get all devices");

    let devices = Device::read_all()?;

    debug!("got devices");

    Ok(Json(json!(devices)))
}
