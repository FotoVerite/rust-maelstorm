use crate::{message::ReplyBody, storage::Storage};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn handle_id_gen(
    src: String,
    dest: String,
    msg_id: u64,
    storage: Arc<Mutex<Storage>>,
) -> anyhow::Result<String> {
    let reply = ReplyBody::GenerateOk {
        in_reply_to: msg_id,
        id: storage.lock().await.boxed_next_id()().to_string(),
    };

    let response = serde_json::json!({
        "src": dest,
        "dest": src,
        "body": reply,
    });
    let json = serde_json::to_string(&response)?;
    Ok(json)
}