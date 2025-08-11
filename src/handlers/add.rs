use tokio::sync::mpsc::Sender;

use crate::{message::ReplyBody, storage::Storage};

pub async fn handle_add(
    src: String,
    dest: String,
    msg_id: u64,
    storage: &mut Storage,
    delta: u64,
    tx: Sender<String>
) -> anyhow::Result<()> {
    storage.process_add(delta, src.clone(), msg_id).await;
    let reply = ReplyBody::AddOk {
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
