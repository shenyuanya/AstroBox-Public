use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::{
    collections::VecDeque,
    sync::{Arc, Weak}, time::Duration,
};
use tokio::sync::{oneshot, Mutex, Notify};

pub static CHANNEL_PRIORITY: Lazy<RwLock<Vec<Channel>>> =
    Lazy::new(|| RwLock::new(vec![Channel::Mass, Channel::Pb, Channel::NetWork]));

pub fn set_channel_priority(priority: Vec<Channel>) {
    *CHANNEL_PRIORITY.write() = priority;
}

fn channel_priority_index(channel: Channel) -> usize {
    let guard = CHANNEL_PRIORITY.read();
    let len = guard.len();
    guard.iter().position(|&c| c == channel).unwrap_or(len)
}

use super::{
    device::{MiWearDevice, REQ_TIMEOUT},
    packet::{Channel, OpCode},
};

#[derive(Debug)]
pub enum CommandKind {
    Send,
    WaitAck,
    RegisterAck { unlocked: bool },
}

pub enum CommandResponse {
    Done,
    AckReceiver(oneshot::Receiver<()>),
}

pub struct Command {
    pub channel: Channel,
    pub op: OpCode,
    pub payload: Vec<u8>,
    pub kind: CommandKind,
    pub timeout: Option<std::time::Duration>,
    pub responder: oneshot::Sender<anyhow::Result<CommandResponse>>,
}

pub struct CommandPool {
    queue: Mutex<VecDeque<Command>>,
    notify: Notify,
    device: Weak<MiWearDevice>,
}

impl CommandPool {
    pub fn new(device: Weak<MiWearDevice>) -> Arc<Self> {
        let pool = Arc::new(Self {
            queue: Mutex::new(VecDeque::new()),
            notify: Notify::new(),
            device,
        });
        let worker = pool.clone();
        tokio::spawn(async move {
            worker.run().await;
        });
        pool
    }

    pub async fn push(&self, cmd: Command) {
        let mut q = self.queue.lock().await;

        log::info!("[CommandPool] Push new command: channel={} timeout={}", cmd.channel as u8, cmd.timeout.unwrap_or(Duration::from_secs(99999)).as_millis());

        let cmd_prio = channel_priority_index(cmd.channel);
        let pos = q
            .iter()
            .rposition(|c| channel_priority_index(c.channel) <= cmd_prio)
            .map_or(0, |i| i + 1);
        q.insert(pos, cmd);

        self.notify.notify_one();
    }

    async fn pop(&self) -> Command {
        loop {
            if let Some(cmd) = self.queue.lock().await.pop_front() {
                return cmd;
            }
            self.notify.notified().await;
        }
    }

    async fn run(self: Arc<Self>) {
        loop {
            let cmd = self.pop().await;
            if let Some(device) = self.device.upgrade() {
                let _ = self.process(device, cmd).await;
            } else {
                break;
            }
        }
    }

    async fn process(&self, device: Arc<MiWearDevice>, cmd: Command) -> anyhow::Result<()> {
        match cmd.kind {
            CommandKind::Send => {
                let frame = device
                    .build_frame(cmd.channel, cmd.op, &cmd.payload)
                    .await?
                    .1;
                let _guard = device.send_lock.lock().await;
                device.send_fragments(frame).await?;
                let _ = cmd.responder.send(Ok(CommandResponse::Done));
            }
            CommandKind::WaitAck => {
                let (_seq, frame) = device
                    .build_frame(cmd.channel, cmd.op, &cmd.payload)
                    .await?;
                let (tx, rx) = oneshot::channel();
                device.pending_ack.insert((), tx);
                let _guard = device.send_lock.lock().await;
                device.send_fragments(frame).await?;
                match tokio::time::timeout(cmd.timeout.unwrap_or(REQ_TIMEOUT), rx).await {
                    Ok(Ok(())) => {
                        let _ = cmd.responder.send(Ok(CommandResponse::Done));
                    }
                    Ok(Err(e)) => {
                        let _ = cmd.responder.send(Err(anyhow::anyhow!(e)));
                    }
                    Err(_) => {
                        let _ = cmd.responder.send(Err(anyhow::anyhow!("Ack timeout")));
                    }
                }
            }
            CommandKind::RegisterAck { unlocked } => {
                let (_seq, frame) = device
                    .build_frame(cmd.channel, cmd.op, &cmd.payload)
                    .await?;
                let (tx, rx) = oneshot::channel();
                device.pending_ack.insert((), tx);
                if !unlocked {
                    let _guard = device.send_lock.lock().await;
                    device.send_fragments(frame).await?;
                } else {
                    device.send_fragments(frame).await?;
                }
                let _ = cmd.responder.send(Ok(CommandResponse::AckReceiver(rx)));
            }
        }
        Ok(())
    }

    pub async fn to_json_table(&self) -> Vec<serde_json::Value> {
        log::info!("to_json_table");
        let q = self.queue.lock().await;
        q.iter()
            .enumerate()
            .map(|(idx, cmd)| {
                serde_json::json!({
                    "Packet": {
                        "label": format!("{:?} {:?}", cmd.channel, cmd.op),
                    },
                    "Info": {
                        "label": format!("CMD #{}", idx + 1),
                        "status": format!("{:?}", cmd.kind),
                    },
                    "Timeout": {
                        "label": cmd.timeout
                            .map(|d| format!("Timeout: {}ms", d.as_millis()))
                            .unwrap_or_else(|| "No timeout".to_string()),
                    },
                    "Status": {
                        "label": "Pending",
                    }
                })
            })
            .collect()
    }
}
