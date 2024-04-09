use crate::config::Config;
use crate::storage::Device;
use dashmap::DashMap;
use ipnetwork::IpNetwork;
use std::fmt::Display;
use time::{Duration, Instant};
use tokio::sync::broadcast::Sender;
use tracing::{debug, error, trace};

pub type StatusMap = DashMap<String, Value>;

#[derive(Debug, Clone)]
pub struct Value {
    pub ip: IpNetwork,
    pub eta: i64,
    pub online: bool,
}

pub async fn spawn(
    tx: Sender<BroadcastCommand>,
    config: &Config,
    device: Device,
    uuid: String,
    ping_map: &StatusMap,
) {
    let timer = Instant::now();
    let payload = [0; 8];

    let mut msg: Option<BroadcastCommand> = None;
    while msg.is_none() {
        let ping = surge_ping::ping(device.ip.ip(), &payload).await;

        if let Err(ping) = ping {
            let ping_timeout = matches!(ping, surge_ping::SurgeError::Timeout { .. });
            if !ping_timeout {
                error!("{}", ping.to_string());
                msg = Some(BroadcastCommand::error(uuid.clone()));
            }
            if timer.elapsed() >= Duration::minutes(config.pingtimeout) {
                msg = Some(BroadcastCommand::timeout(uuid.clone()));
            }
        } else {
            let (_, duration) = ping
                .map_err(|err| error!("{}", err.to_string()))
                .expect("fatal error");
            debug!("ping took {:?}", duration);
            msg = Some(BroadcastCommand::success(uuid.clone()));
        };
    }

    trace!(?msg);

    let msg = msg.expect("fatal error");

    let _ = tx.send(msg.clone());
    if msg.command == BroadcastCommands::Success {
        if timer.elapsed().whole_seconds() > config.pingthreshold {
            let newtimes = if let Some(mut oldtimes) = device.times {
                oldtimes.push(timer.elapsed().whole_seconds());
                oldtimes
            } else {
                vec![timer.elapsed().whole_seconds()]
            };

            let updatedev = Device {
                id: device.id,
                mac: device.mac,
                broadcast_addr: device.broadcast_addr,
                ip: device.ip,
                times: Some(newtimes),
            };
            updatedev.write().unwrap();
        }

        ping_map.alter(&uuid, |_, v| Value {
            ip: v.ip,
            eta: v.eta,
            online: true,
        });

        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
    trace!("remove {} from ping_map", uuid);
    ping_map.remove(&uuid);
}

#[derive(Clone, Debug, PartialEq)]
pub enum BroadcastCommands {
    Success,
    Timeout,
    Error,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BroadcastCommand {
    pub uuid: String,
    pub command: BroadcastCommands,
}

impl Display for BroadcastCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = match self.command {
            BroadcastCommands::Success => "start",
            BroadcastCommands::Timeout => "timeout",
            BroadcastCommands::Error => "error",
        };

        f.write_str(format!("{prefix}_{}", self.uuid).as_str())
    }
}

impl BroadcastCommand {
    pub fn success(uuid: String) -> Self {
        Self {
            uuid,
            command: BroadcastCommands::Success,
        }
    }

    pub fn timeout(uuid: String) -> Self {
        Self {
            uuid,
            command: BroadcastCommands::Timeout,
        }
    }

    pub fn error(uuid: String) -> Self {
        Self {
            uuid,
            command: BroadcastCommands::Error,
        }
    }
}
