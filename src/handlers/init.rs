use tokio::sync::mpsc::Sender;

use crate::message::ReplyBody;

pub async fn handle_init(
    src: String,
    dest: String,
    msg_id: u64,
    tx: Sender<String>,
) -> anyhow::Result<()> {
    let reply = ReplyBody::InitOk {
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
