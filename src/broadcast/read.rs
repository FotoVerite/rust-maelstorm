use crate::message::{Body};

pub fn send_read(
    src: &String,
    dest: String,
    msg_id: u64,
) -> anyhow::Result<String> {
    let reply = Body::Read {
        msg_id: msg_id,
        key: None
    };

    let response = serde_json::json!({
        "src": src,
        "dest": dest,
        "body": reply,
    });
    let json = serde_json::to_string(&response)?;

    Ok(json)
}
