use std::time::Duration;

use tokio::sync::mpsc::{Receiver, Sender};

use crate::{broadcast::broadcast::send_broadcast, storage::NodeId};

pub enum BroadcastCommand {
    Broadcast {
        dest: String,
        msg_id: u64,
        message: u64,
    },
}

pub async fn broadcast_message(
    mut rx: Receiver<BroadcastCommand>,
    tx: Sender<String>,
    id: NodeId,
) -> anyhow::Result<()> {
    loop {
        let id = {
            let guard = id.lock().await;
            if guard.is_none() {
                drop(guard);
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }
            guard.clone().unwrap()
        };

        // Always handle 'init' globally
        while let Some(command) = rx.recv().await {
            match command {
                BroadcastCommand::Broadcast {
                    dest,
                    msg_id,
                    message,
                } => send_broadcast(id.clone(), dest, msg_id, message, tx.clone()).await?,
            }
        }
    }
}
