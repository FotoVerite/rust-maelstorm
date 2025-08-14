use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{
    message::{ReadMessage, ReplyBody},
    storage::Storage,
};

pub async fn handle_read(
    src: String,
    dest: String,
    msg_id: u64,
    storage: Arc<Mutex<Storage>>,
) -> anyhow::Result<String> {
    let guard: tokio::sync::MutexGuard<'_, Storage> = storage.lock().await;
    let messages_vec = guard.values.values();
    drop(guard);

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

    Ok(json)
}

pub async fn handle_g_counter_read(
    src: String,
    dest: String,
    msg_id: u64,
    storage: Arc<Mutex<Storage>>,
) -> anyhow::Result<String> {
    let mut guard: tokio::sync::MutexGuard<'_, Storage> = storage.lock().await;
    let message = guard.g_counter.local_value();
    let reply = ReplyBody::ReadOk {
        in_reply_to: msg_id,
        messages: ReadMessage::Single(message),
    };
    let response = serde_json::json!({
        "src": dest,
        "dest": src,
        "body": reply,
    });
    let json = serde_json::to_string(&response)?;

    Ok(json)
}
