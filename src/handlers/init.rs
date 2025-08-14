use crate::message::ReplyBody;

pub async fn handle_init(
    src: String,
    dest: String,
    msg_id: u64,
) -> anyhow::Result<String> {
    let reply = ReplyBody::InitOk {
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
