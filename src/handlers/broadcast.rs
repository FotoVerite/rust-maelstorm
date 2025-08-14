use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{
    message::{BroadcastMessage, ReplyBody},
    storage::Storage,
};

pub async fn handle_broadcast(
    src: String,
    dest: String,
    msg_id: u64,
    storage: Arc<Mutex<Storage>>,
    message: BroadcastMessage,
) -> anyhow::Result<String> {
    let src_for_response = src.clone();
    let mut guard = storage.lock().await;
    match message {
        BroadcastMessage::Single(value) => {
            if guard.values.update_store_data(value) {
                Some(value)
            } else {
                None
            }
        }
        _ => None,
    };

    // 3. Immediate broadcast_ok reply
    let reply = ReplyBody::BroadcastOk {
        in_reply_to: msg_id,
    };
    let response = serde_json::json!({
        "src": dest,
        "dest": src_for_response,
        "body": reply,
    });

    Ok(serde_json::to_string(&response)?)
}

pub async fn handle_broadcast_g_counter(
    src: String,
    dest: String,
    msg_id: u64,
    storage: Arc<Mutex<Storage>>,
    message: BroadcastMessage,
) -> anyhow::Result<String> {
    match message {
        BroadcastMessage::Hashmap(value) => {
            tokio::spawn(async move {
                let mut storage = storage.lock().await;
                let _ = storage.g_counter.update_counter(value);
            });
        }
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

    Ok(json)
}
