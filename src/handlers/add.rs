use std::sync::{Arc};

use tokio::sync::Mutex;

use crate::{message::ReplyBody, storage::Storage};

pub async fn handle_add(
    src: String,
    dest: String,
    msg_id: u64,
    storage: Arc<Mutex<Storage>>,
    delta: u64,
) -> anyhow::Result<String> {
    tokio::spawn(async move {
        let mut storage = storage.lock().await;
        let _ = storage.g_counter.process_add(delta).await;
    });

    let reply = ReplyBody::AddOk {
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
