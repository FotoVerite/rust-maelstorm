use tokio::sync::mpsc::Sender;

use crate::{message::{ReadMessage, ReplyBody}, storage::Storage};

pub async fn handle_read(
    src: String,
    dest: String,
    msg_id: u64,
    storage: &Storage,
    tx: Sender<String>,
) -> anyhow::Result<()> {
    let messages_vec = storage.values();

    let reply_body = serde_json::json!({
        "type": "read_ok",
        "in_reply_to": msg_id,
        "messages": messages_vec,
    });

    let response = serde_json::json!({
        "src": dest,
        "dest": src,
        "body": reply_body,
    });
    let json = serde_json::to_string(&response)?;

    Ok(tx.send(json).await?)
}


pub async fn handle_g_counter_read(
    src: String,
    dest: String,
    msg_id: u64,
    storage: &Storage,
    tx: Sender<String>,
) -> anyhow::Result<()> {
    let reply = ReplyBody::ReadOk {
        in_reply_to: msg_id,
        messages: ReadMessage::Single(storage.g_counter_value()),
    };
    let response = serde_json::json!({
        "src": dest,
        "dest": src,
        "body": reply,
    });
    let json = serde_json::to_string(&response)?;

    Ok(tx.send(json).await?)
}
