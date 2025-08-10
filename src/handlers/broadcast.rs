use tokio::sync::mpsc::Sender;

use crate::{
    message::{ReplyBody},
    storage::Storage,
};

pub async fn handle_broadcast (
    src: String,
    dest: String,
    msg_id: u64,
    storage: &mut Storage,
    message: u64,
    tx: Sender<String>,
) -> anyhow::Result<()> {
    storage.update_values(src.clone(), message);
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
