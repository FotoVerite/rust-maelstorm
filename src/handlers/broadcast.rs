use tokio::sync::mpsc::Sender;

use crate::{
    message::{BroadcastMessage, ReplyBody},
    storage::Storage,
};

pub async fn handle_broadcast(
    src: String,
    dest: String,
    msg_id: u64,
    storage: &mut Storage,
    message: BroadcastMessage,
    tx: Sender<String>,
) -> anyhow::Result<()> {
    match message {
        BroadcastMessage::Single(value) => {storage.update_values(src.clone(), value)},
        BroadcastMessage::Hashmap(values ) => storage.update_counter(values),
        _ => {}

    }
    let reply = ReplyBody::BroadcastOk {
        in_reply_to: msg_id,
    };
    let response = serde_json::json!({
        "src": dest,
        "dest": src,
        "body": reply,
    });
    let json = serde_json::to_string(&response)?;

    Ok(tx.send(json).await?)
}

pub async fn handle_broadcast_g_counter(
    src: String,
    dest: String,
    msg_id: u64,
    storage: &mut Storage,
    message: BroadcastMessage,
    tx: Sender<String>,
) -> anyhow::Result<()> {
    match message {
        BroadcastMessage::Hashmap(value) => storage.update_counter(value),
        _ => {}
    }
    let reply = ReplyBody::BroadcastOk {
        in_reply_to: msg_id,
    };
    let response = serde_json::json!({
        "src": dest,
        "dest": src,
        "body": reply,
    });
    let json = serde_json::to_string(&response)?;

    Ok(tx.send(json).await?)
}
