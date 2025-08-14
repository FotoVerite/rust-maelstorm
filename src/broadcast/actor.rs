use std::{sync::Arc, time::Duration};

use tokio::sync::{
    Mutex,
    mpsc::{Receiver, Sender},
};

use crate::{
    broadcast::{broadcast::send_broadcast, cas::send_cas, read::send_read},
    message::BroadcastMessage,
    storage::Storage,
};

#[derive(Debug, Clone)]
pub enum BroadcastCommand {
    Broadcast {
        dest: String,
        msg_id: u64,
        message: BroadcastMessage,
    },

    Cas {
        dest: String,
        msg_id: u64,
        from: u64,
        to: u64,
        create_if_not_exists: bool,
    },

    Read {
        dest: String,
        msg_id: u64,
    },
}
pub async fn broadcast_message(
    mut rx: Receiver<BroadcastCommand>,
    storage: Arc<Mutex<Storage>>,
    tx: Sender<String>,
) -> anyhow::Result<()> {
    // Wait until the node is initialized before we start processing commands.
    let id = {
        let mut guard = storage.lock().await;

        while guard.node_id.read().unwrap().is_none() {
            drop(guard);
            tokio::time::sleep(Duration::from_millis(100)).await;
            guard = storage.lock().await;
        }

        guard.get_node_id()
    };
    while let Some(command) = rx.recv().await {
        let message_result = match command {
            BroadcastCommand::Broadcast {
                dest,
                msg_id,
                message,
            } => send_broadcast(&id, dest, msg_id, message),
            BroadcastCommand::Cas {
                dest,
                msg_id,
                from,
                to,
                create_if_not_exists,
            } => send_cas(&id, dest, msg_id, from, to, create_if_not_exists),
            BroadcastCommand::Read { dest, msg_id } => send_read(&id, dest, msg_id),
        };

        let message = match message_result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("Error sending message: {:?}", e);
                continue; // Log error and continue to next command
            }
        };

        if tx.send(message).await.is_err() {
            eprintln!("Error sending message to stdout channel, receiver likely closed.");
            break; // Main stdout channel has closed, we can exit.
        }
    }
    Ok(())
}
