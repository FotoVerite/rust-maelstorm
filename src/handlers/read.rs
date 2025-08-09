use tokio::sync::mpsc::Sender;

use crate::{message::ReplyBody, storage::Storage};

pub async fn handle_read(
    src: String,
    dest: String,
    msg_id: u64,
    storage: &Storage,
    tx: Sender<String>,
) -> anyhow::Result<()> {
    let reply = ReplyBody::ReadOk {
        in_reply_to: msg_id,
        messages: storage.values(),
    };
    let response = serde_json::json!({
        "src": dest,
        "dest": src,
        "body": reply,
    });
    let json = serde_json::to_string(&response)?;

    Ok(tx.send(json).await?)
}
