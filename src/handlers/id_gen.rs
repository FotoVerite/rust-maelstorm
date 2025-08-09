use tokio::sync::mpsc::Sender;

use crate::{
    message::ReplyBody, state::State,
};

pub async fn handle_id_gen (
    src: String,
    dest: String,
    msg_id: u64,
    state: &mut State,
    tx: Sender<String>,
) -> anyhow::Result<()> {
    let reply = ReplyBody::GenerateOk  {
        in_reply_to: msg_id,
        id: state.next_id()
    };

    let response = serde_json::json!({
        "src": dest,
        "dest": src,
        "body": reply,
    });
    let json = serde_json::to_string(&response)?;

    Ok(tx.send(json).await?)
}
