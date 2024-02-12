use std::str::FromStr;
use std::net::IpAddr;
use std::sync::Arc;

use axum::extract::ws::WebSocket;
use axum::extract::ws::Message;
use dashmap::DashMap;
use sqlx::PgPool;
use time::{Duration, Instant};
use tokio::sync::broadcast::Sender;
use tracing::{debug, error, trace};
use crate::AppState;
use crate::config::Config;
use crate::db::Device;

pub type StatusMap = DashMap<String, Value>;

#[derive(Debug, Clone)]
pub struct Value {
    pub ip: String,
    pub online: bool
}

pub async fn spawn(tx: Sender<BroadcastCommands>, config: &Config, device: Device, uuid: String, ping_map: &StatusMap, db: &PgPool) {
    let timer = Instant::now();
    let payload = [0; 8];

    let ping_ip = IpAddr::from_str(&device.ip).expect("bad ip");

    let mut msg: Option<BroadcastCommands> = None;
    while msg.is_none() {
        let ping = surge_ping::ping(
            ping_ip,
            &payload
        ).await;

        if let Err(ping) = ping {
            let ping_timeout = matches!(ping, surge_ping::SurgeError::Timeout { .. });
            if !ping_timeout {
                error!("{}", ping.to_string());
                msg = Some(BroadcastCommands::Error(uuid.clone()));
            }
            if timer.elapsed() >= Duration::minutes(config.pingtimeout) {
                msg = Some(BroadcastCommands::Timeout(uuid.clone()));
            }
        } else {
            let (_, duration) = ping.map_err(|err| error!("{}", err.to_string())).expect("fatal error");
            debug!("ping took {:?}", duration);
            msg = Some(BroadcastCommands::Success(uuid.clone()));
        };
    }

    let msg = msg.expect("fatal error");

    let _ = tx.send(msg.clone());
    if let BroadcastCommands::Success(..) = msg {
        sqlx::query!(
            r#"
            UPDATE devices
            SET times = array_append(times, $1)
            WHERE id = $2;
            "#,
            timer.elapsed().whole_seconds(),
            device.id
        ).execute(db).await.unwrap();
        ping_map.insert(uuid.clone(), Value { ip: device.ip.clone(), online: true });
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
    trace!("remove {} from ping_map", uuid);
    ping_map.remove(&uuid);
}

#[derive(Clone, Debug, PartialEq)]
pub enum BroadcastCommands {
    Success(String),
    Timeout(String),
    Error(String),
}

pub async fn status_websocket(mut socket: WebSocket, state: Arc<AppState>) {
    trace!("wait for ws message (uuid)");
    let msg = socket.recv().await;
    let uuid = msg.unwrap().unwrap().into_text().unwrap();

    trace!("Search for uuid: {}", uuid);

    let eta = get_eta(&state.db).await;
    let _ = socket.send(Message::Text(format!("eta_{eta}_{uuid}"))).await;

    let device_exists = state.ping_map.contains_key(&uuid);
    if device_exists {
        let _ = socket.send(process_device(state.clone(), uuid).await).await;
    } else {
        debug!("didn't find any device");
        let _ = socket.send(Message::Text(format!("notfound_{uuid}"))).await;
    };

    let _ = socket.close().await;
}

async fn get_eta(db: &PgPool) -> i64 {
    let query = sqlx::query!(
        r#"SELECT times FROM devices;"#
    ).fetch_one(db).await.unwrap();

    let times = match query.times {
        None => { vec![0] },
        Some(t) => t,
    };
    times.iter().sum::<i64>() / i64::try_from(times.len()).unwrap()

}

async fn process_device(state: Arc<AppState>, uuid: String) -> Message {
    let pm = state.ping_map.clone().into_read_only();
    let device = pm.get(&uuid).expect("fatal error");
    debug!("got device: {} (online: {})", device.ip, device.online);
    if device.online {
        debug!("already started");
        Message::Text(format!("start_{uuid}"))
    } else {
        loop {
            trace!("wait for tx message");
            let message = state.ping_send.subscribe().recv().await.expect("fatal error");
            trace!("got message {:?}", message);
            return match message {
                BroadcastCommands::Success(msg_uuid) => {
                    if msg_uuid != uuid { continue; }
                    trace!("message == uuid success");
                    Message::Text(format!("start_{uuid}"))
                },
                BroadcastCommands::Timeout(msg_uuid) => {
                    if msg_uuid != uuid { continue; }
                    trace!("message == uuid timeout");
                    Message::Text(format!("timeout_{uuid}"))
                },
                BroadcastCommands::Error(msg_uuid) => {
                    if msg_uuid != uuid { continue; }
                    trace!("message == uuid error");
                    Message::Text(format!("error_{uuid}"))
                }
            }
        }
    }
}
