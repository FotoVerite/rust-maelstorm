use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncWriteExt};

use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::handlers::add::handle_add;
use crate::handlers::broadcast::handle_broadcast;
use crate::handlers::broadcast_ok::handle_broadcast_ok;
use crate::handlers::cas_ok::handle_cas_ok;
use crate::handlers::echo::handle_echo;
use crate::handlers::error::handle_error;
use crate::handlers::id_gen::handle_id_gen;
use crate::handlers::init::handle_init;
use crate::handlers::read::{handle_g_counter_read, handle_read};
use crate::handlers::topology::handle_topology;
use crate::message::{Body, Message};
use crate::storage::Storage;

pub mod broadcast;
pub mod handlers;
pub mod message;
mod snowflake;
pub mod storage;
pub mod workers;

pub trait Handler {
    /// Handle an incoming message, possibly mutating state, and produce zero or more responses.
    fn handle(&mut self, msg: &Message, state: &mut Storage) -> Vec<Message>;
}

pub async fn process_message_line(
    line: String,
    storage: Arc<Mutex<Storage>>,
    processed: &mut HashMap<(String, u64), Option<String>>,
    tx: Sender<String>,
) -> anyhow::Result<()> {
    let workload = "broadcast";

    let msg: Message = serde_json::from_str(&line).expect("Invalid JSON");

    // Always handle 'init' globally

    let src = msg.src;

    let dest = msg.dest;
    let body = msg.body;
    let key = (src.clone(), *body.cache_id());

    if let Some(Some(json)) = processed.get(&key) {
        tx.send(json.clone()).await?;
        return Ok(())
    }
    // // Dispatch to specific handlers based on workload
    let json: Option<String> = match body {
        Body::Init {
            msg_id,
            node_id,
            node_ids: _,
            workload: _,
        } => {
            let mut guard = storage.lock().await;
            guard.set_id(&node_id).await;
            Some(handle_init(src, dest, msg_id).await?)
        }
        Body::Add { msg_id, delta } => Some(handle_add(src, dest, msg_id, storage, delta).await?),
        Body::Cas { .. } => None,
        Body::CasOk { in_reply_to } => {
            handle_cas_ok(src, in_reply_to, storage).await?;
            None
        }
        Body::Broadcast { msg_id, message } => {
            Some(handle_broadcast(src, dest, msg_id, storage, message).await?)
        }
        Body::BroadcastOk { in_reply_to } => {
            handle_broadcast_ok(src, dest, in_reply_to, storage).await?;
            None
        }
        Body::Echo { msg_id, echo } => Some(handle_echo(src, dest, msg_id, echo)?),
        Body::Error {
            in_reply_to,
            code: _,
            text: _,
        } => {
            handle_error(in_reply_to, storage).await?;
            None
        }

        Body::Generate { msg_id } => Some(handle_id_gen(src, dest, msg_id, storage).await?),
        Body::Read { msg_id, key: _ } => match workload {
            "g-counter" => Some(handle_g_counter_read(src, dest, msg_id, storage).await?),
            _ => Some(handle_read(src, dest, msg_id, storage).await?),
        },
        Body::Topology { msg_id, topology } => {
            Some(handle_topology(src, dest, msg_id, topology, storage).await?)
        }
    };
    processed.insert(key, json.clone());
    if let Some(json) = json {
        Ok(tx.send(json).await?)
    } else {
        Ok(())
    }
}

pub async fn write_stdout<W: tokio::io::AsyncWrite + Unpin>(
    mut writer: W,
    mut rx: Receiver<String>,
) -> anyhow::Result<()> {
    while let Some(mut msg) = rx.recv().await {
        msg.push('\n');
        writer.write_all(msg.as_bytes()).await?;
        writer.flush().await?;
    }
    Ok(())
}
