use tokio::sync::mpsc::Sender;

use crate::{
    message::{Body},
};

pub async fn send_broadcast (
    src: String,
    dest: String,
    msg_id: u64,
    message: u64,
    tx: Sender<String>,
) -> anyhow::Result<()> {
    let reply = Body::Broadcast { msg_id: msg_id, message: message };
    
    let response = serde_json::json!({
        "src": src,
        "dest": dest,
        "body": reply,
    });
    let json = serde_json::to_string(&response)?;

    Ok(tx.send(json).await?)
}
