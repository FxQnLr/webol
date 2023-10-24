use std::borrow::Cow;
use std::sync::Arc;

use axum::{extract::{WebSocketUpgrade, ws::WebSocket, State}, response::Response};
use tokio::sync::broadcast::{Sender};
use tracing::{debug, error, trace};

use crate::{error::WebolError, AppState};

pub async fn spawn(tx: Sender<String>) -> Result<(), WebolError> {
    let payload = [0; 8];

    let mut cont = true;
    while cont {
        let ping = surge_ping::ping(
            "127.0.0.1".parse().map_err(WebolError::IpParse)?,
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
            tx.send("Got ping".to_string()).unwrap();
        };
    }

    Ok(())
}

// TODO: Status to routes, websocket here
pub async fn ws_ping(State(state): State<Arc<AppState>>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state.ping_send.clone()))
}

// FIXME: Handle commands through enum
async fn handle_socket(mut socket: WebSocket, tx: Sender<String>) {
    // TODO: Understand Cow
    while let message = tx.subscribe().recv().await.unwrap() {
        trace!("GOT = {}", message);
        if &message == "Got ping" {
            break;
        }
    };
    match socket.send(axum::extract::ws::Message::Close(Some(axum::extract::ws::CloseFrame { code: 4000, reason: Cow::Owned("started".to_owned()) }))).await.map_err(WebolError::Axum) {
        Ok(..) => (),
        Err(err) => { error!("Server Error: {:?}", err) }
    };
}