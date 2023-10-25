use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{ws::WebSocket};
use axum::extract::ws::Message;
use tokio::sync::broadcast::{Sender};
use tokio::sync::Mutex;
use tracing::{debug, error, trace, warn};

use crate::error::WebolError;

pub async fn spawn(tx: Sender<String>, ip: String) -> Result<(), WebolError> {
    let payload = [0; 8];

    let mut cont = true;
    while cont {
        let ping = surge_ping::ping(
            ip.parse().map_err(WebolError::IpParse)?,
            &payload
        ).await;

        if let Err(ping) = ping {
            cont = matches!(ping, surge_ping::SurgeError::Timeout { .. });

            // debug!("{}", cont);
            
            if !cont {
                return Err(ping).map_err(WebolError::Ping)
            }

        } else {
            let (_, duration) = ping.unwrap();
            debug!("Ping took {:?}", duration);
            cont = false;
            // FIXME: remove unwrap
            // FIXME: if error: SendError because no listener, then handle the entry directly
            tx.send(ip.clone());
        };
    }

    Ok(())
}

// FIXME: Handle commands through enum
pub async fn status_websocket(mut socket: WebSocket, tx: Sender<String>, ping_map: Arc<Mutex<HashMap<String, (String, bool)>>>) {
    warn!("{:?}", ping_map);

    let mut uuid: Option<String> = None;

    trace!("wait for ws message (uuid)");
    let msg = socket.recv().await;
    uuid = Some(msg.unwrap().unwrap().into_text().unwrap());

    let uuid = uuid.unwrap();

    trace!("Search for uuid: {:?}", uuid);

    let device = ping_map.lock().await.get(&uuid).unwrap().to_owned();

    trace!("got device: {:?}", device);

    match device.1 {
        true => {
            debug!("already started");
            socket.send(Message::Text(format!("start_{}", uuid))).await.unwrap();
            socket.close().await.unwrap();
        },
        false => {
            let ip = device.0.to_owned();
            let mut i = 0;
            loop{
                trace!("{}", i);
                // TODO: Check if older than 10 minutes, close if true
                trace!("wait for tx message");
                let message = tx.subscribe().recv().await.unwrap();
                trace!("GOT = {}", message);
                if message == ip {
                    trace!("message == ip");
                    break;
                }
                i += 1;
            };

            socket.send(Message::Text(format!("start_{}", uuid))).await.unwrap();
            socket.close().await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            ping_map.lock().await.remove(&uuid);
            warn!("{:?}", ping_map);
        }
    }
}