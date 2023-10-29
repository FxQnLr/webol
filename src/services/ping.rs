use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{ws::WebSocket};
use axum::extract::ws::{CloseFrame, Message};
use tokio::sync::broadcast::{Sender};
use tokio::sync::Mutex;
use tracing::{debug, trace, warn};

use crate::error::WebolError;

pub type PingMap = Arc<Mutex<HashMap<String, (String, bool)>>>;

pub async fn spawn(tx: Sender<BroadcastCommands>, ip: String, uuid: String, ping_map: PingMap) -> Result<(), WebolError> {
    let payload = [0; 8];

    // TODO: Better while
    let mut cont = true;
    while cont {
        let ping = surge_ping::ping(
            ip.parse().map_err(WebolError::IpParse)?,
            &payload
        ).await;

        if let Err(ping) = ping {
            cont = matches!(ping, surge_ping::SurgeError::Timeout { .. });
            if !cont {
                return Err(ping).map_err(WebolError::Ping)
            }
        } else {
            let (_, duration) = ping.unwrap();
            debug!("Ping took {:?}", duration);
            cont = false;
            handle_broadcast_send(&tx, ip.clone(), ping_map.clone(), uuid.clone()).await;
        };
    }

    Ok(())
}

async fn handle_broadcast_send(tx: &Sender<BroadcastCommands>, ip: String, ping_map: PingMap, uuid: String) {
    debug!("sending pingsuccess message");
    ping_map.lock().await.insert(uuid.clone(), (ip.clone(), true));
    let _ = tx.send(BroadcastCommands::PingSuccess(ip));
    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    trace!("remove {} from ping_map", uuid);
    ping_map.lock().await.remove(&uuid);
}

#[derive(Clone, Debug)]
pub enum BroadcastCommands {
    PingSuccess(String)
}

pub async fn status_websocket(mut socket: WebSocket, tx: Sender<BroadcastCommands>, ping_map: PingMap) {
    warn!("{:?}", ping_map);

    trace!("wait for ws message (uuid)");
    let msg = socket.recv().await;
    let uuid = msg.unwrap().unwrap().into_text().unwrap();

    trace!("Search for uuid: {:?}", uuid);

    // TODO: Handle Error
    let device = ping_map.lock().await.get(&uuid).unwrap().to_owned();

    trace!("got device: {:?}", device);

    match device.1 {
        true => {
            debug!("already started");
            // socket.send(Message::Text(format!("start_{}", uuid))).await.unwrap();
            // socket.close().await.unwrap();
            socket.send(Message::Close(Some(CloseFrame { code: 4001, reason: Cow::from(format!("start_{}", uuid)) }))).await.unwrap();
        },
        false => {
            let ip = device.0.to_owned();
            loop{
                trace!("wait for tx message");
                let message = tx.subscribe().recv().await.unwrap();
                trace!("GOT = {:?}", message);
                // if let BroadcastCommands::PingSuccess(msg_ip) = message {
                //     if msg_ip == ip {
                //         trace!("message == ip");
                //         break;
                //     }
                // }
                let BroadcastCommands::PingSuccess(msg_ip) = message;
                if msg_ip == ip {
                    trace!("message == ip");
                    break;
                }
            };

            socket.send(Message::Close(Some(CloseFrame { code: 4000, reason: Cow::from(format!("start_{}", uuid)) }))).await.unwrap();
            // socket.send(Message::Text(format!("start_{}", uuid))).await.unwrap();
            // socket.close().await.unwrap();
            warn!("{:?}", ping_map);
        }
    }
}