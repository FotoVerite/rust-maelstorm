use crate::{
    message::{ReplyBody},
};

pub fn handle_echo(
    src: String,
    dest: String,
    msg_id: u64,
    echo: String,
) -> anyhow::Result<String> {
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

    Ok(json)
}
