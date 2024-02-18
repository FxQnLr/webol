use crate::services::ping::BroadcastCommand;
use crate::AppState;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::Response;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{debug, trace};

pub async fn status(State(state): State<Arc<AppState>>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(move |socket| websocket(socket, state))
}

pub async fn websocket(mut socket: WebSocket, state: Arc<AppState>) {
    trace!("wait for ws message (uuid)");
    let msg = socket.recv().await;
    let uuid = msg.unwrap().unwrap().into_text().unwrap();

    trace!("Search for uuid: {}", uuid);

    let eta = get_eta(&state.db).await;
    let _ = socket
        .send(Message::Text(format!("eta_{eta}_{uuid}")))
        .await;

    let device_exists = state.ping_map.contains_key(&uuid);
    if device_exists {
        let _ = socket
            .send(receive_ping_broadcast(state.clone(), uuid).await)
            .await;
    } else {
        debug!("didn't find any device");
        let _ = socket.send(Message::Text(format!("notfound_{uuid}"))).await;
    };

    let _ = socket.close().await;
}

async fn receive_ping_broadcast(state: Arc<AppState>, uuid: String) -> Message {
    let pm = state.ping_map.clone().into_read_only();
    let device = pm.get(&uuid).expect("fatal error");
    debug!("got device: {} (online: {})", device.ip, device.online);
    if device.online {
        debug!("already started");
        Message::Text(BroadcastCommand::success(uuid).to_string())
    } else {
        loop {
            trace!("wait for tx message");
            let message = state
                .ping_send
                .subscribe()
                .recv()
                .await
                .expect("fatal error");
            trace!("got message {:?}", message);

            if message.uuid != uuid {
                continue;
            }
            trace!("message == uuid success");
            return Message::Text(message.to_string());
        }
    }
}

async fn get_eta(db: &PgPool) -> i64 {
    let query = sqlx::query!(r#"SELECT times FROM devices;"#)
        .fetch_one(db)
        .await
        .unwrap();

    let times = if let Some(times) = query.times {
        times
    } else {
        vec![0]
    };

    times.iter().sum::<i64>() / i64::try_from(times.len()).unwrap()
}
