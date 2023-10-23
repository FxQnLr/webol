use std::sync::Arc;

use axum::{extract::{WebSocketUpgrade, ws::WebSocket, State}, response::Response};
use tokio::sync::mpsc::Sender;
use tracing::{debug, error};

use crate::{error::WebolError, AppState};

pub async fn spawn(tx: Sender<String>) -> Result<(), WebolError> {
    let payload = [0; 8];

    let mut cont = true;
    while cont {
        let ping = surge_ping::ping(
            "192.168.178.28".parse().map_err(WebolError::IpParse)?,
            &payload
        ).await;

        if let Err(ping) = ping {
            cont = matches!(ping, surge_ping::SurgeError::Timeout { .. });

            debug!("{}", cont);
            
            if !cont {
                return Err(ping).map_err(WebolError::Ping)
            }

        } else {
            let (_, duration) = ping.unwrap();
            debug!("Ping took {:?}", duration);
            cont = false;
            // FIXME: remove unwrap
            tx.send("Got ping".to_string()).await.unwrap();
        };
    }

    Ok(())
}

pub async fn ws_ping(ws: WebSocketUpgrade, State(_state): State<Arc<AppState>>) -> Response {
    ws.on_upgrade(handle_socket)
}

// FIXME: Handle commands through enum
async fn handle_socket(mut socket: WebSocket) {
    // TODO: Understand Cow

    // match socket.send(axum::extract::ws::Message::Close(Some(CloseFrame { code: 4000, reason: Cow::Owned("started".to_owned()) }))).await.map_err(WebolError::Axum) {
    match socket.send(axum::extract::ws::Message::Text("started".to_string())).await.map_err(WebolError::Axum) {
        Ok(..) => (),
        Err(err) => { error!("Server Error: {:?}", err) }
    };
}
