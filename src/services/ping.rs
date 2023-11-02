use std::sync::Arc;

use axum::extract::{ws::WebSocket};
use axum::extract::ws::Message;
use dashmap::DashMap;
use time::{Duration, Instant};
use tokio::sync::broadcast::{Sender};
use tracing::{debug, error, trace};
use crate::AppState;
use crate::config::SETTINGS;

pub type PingMap = DashMap<String, PingValue>;

#[derive(Debug, Clone)]
pub struct PingValue {
    pub ip: String,
    pub online: bool
}

pub async fn spawn(tx: Sender<BroadcastCommands>, ip: String, uuid: String, ping_map: &PingMap) {
    let timer = Instant::now();
    let payload = [0; 8];

    let mut cont = true;
    while cont {
        let ping = surge_ping::ping(
            ip.parse().expect("bad ip"),
            &payload
        ).await;

        if let Err(ping) = ping {
            cont = matches!(ping, surge_ping::SurgeError::Timeout { .. });
            if !cont {
                error!("{}", ping.to_string());
            }
            if timer.elapsed() >= Duration::minutes(SETTINGS.get_int("pingtimeout").unwrap_or(10)) {
                let _ = tx.send(BroadcastCommands::PingTimeout(uuid.clone()));
                trace!("remove {} from ping_map after timeout", uuid);
                ping_map.remove(&uuid);
                cont = false;
            }
        } else {
            let (_, duration) = ping.map_err(|err| error!("{}", err.to_string())).expect("fatal error");
            debug!("ping took {:?}", duration);
            cont = false;
            handle_broadcast_send(&tx, ip.clone(), ping_map, uuid.clone()).await;
        };
    }
}

async fn handle_broadcast_send(tx: &Sender<BroadcastCommands>, ip: String, ping_map: &PingMap, uuid: String) {
    debug!("send pingsuccess message");
    let _ = tx.send(BroadcastCommands::PingSuccess(uuid.clone()));
    trace!("sent message");
    ping_map.insert(uuid.clone(), PingValue { ip: ip.clone(), online: true });
    trace!("updated ping_map");
    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    debug!("remove {} from ping_map after success", uuid);
    ping_map.remove(&uuid);
}

#[derive(Clone, Debug)]
pub enum BroadcastCommands {
    PingSuccess(String),
    PingTimeout(String)
}

pub async fn status_websocket(mut socket: WebSocket, state: Arc<AppState>) {
    trace!("wait for ws message (uuid)");
    let msg = socket.recv().await;
    let uuid = msg.unwrap().unwrap().into_text().unwrap();

    trace!("Search for uuid: {:?}", uuid);

    let device_exists = state.ping_map.contains_key(&uuid);
    match device_exists {
        true => {
            let _ = socket.send(process_device(state.clone(), uuid).await).await;
        },
        false => {
            debug!("didn't find any device");
            let _ = socket.send(Message::Text(format!("notfound_{}", uuid))).await;
        },
    };

    let _ = socket.close().await;
}

async fn process_device(state: Arc<AppState>, uuid: String) -> Message {
    let pm = state.ping_map.clone().into_read_only();
    let device = pm.get(&uuid).expect("fatal error");
    debug!("got device: {} (online: {})", device.ip, device.online);
    match device.online {
        true => {
            debug!("already started");
            Message::Text(format!("start_{}", uuid))
        },
        false => {
            loop{
                trace!("wait for tx message");
                let message = state.ping_send.subscribe().recv().await.expect("fatal error");
                trace!("got message {:?}", message);
                return match message {
                    BroadcastCommands::PingSuccess(msg_uuid) => {
                        if msg_uuid != uuid { continue; }
                        trace!("message == uuid success");
                        Message::Text(format!("start_{}", uuid))
                    },
                    BroadcastCommands::PingTimeout(msg_uuid) => {
                        if msg_uuid != uuid { continue; }
                        trace!("message == uuid timeout");
                        Message::Text(format!("timeout_{}", uuid))
                    }
                }
            }
        }
    }
}