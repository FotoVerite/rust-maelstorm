use tokio::sync::mpsc::Sender;

use crate::{
    message::{ReplyBody},
};

pub async fn handle_echo(
    src: String,
    dest: String,
    msg_id: u64,
    echo: String,
    tx: Sender<String>,
) -> anyhow::Result<()> {
    let reply = ReplyBody::EchoOk {
        in_reply_to: msg_id,
        echo: echo
    };

    let response = serde_json::json!({
        "src": dest,
        "dest": src,
        "body": reply,
    });
    let json = serde_json::to_string(&response)?;

    Ok(tx.send(json).await?)
}
