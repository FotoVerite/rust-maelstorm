use crate::{
    message::{Body},
};

pub  fn send_cas (
    src: &String,
    dest: String,
    msg_id: u64,
    from: u64, 
    to: u64,
    create_if_not_exists: bool,
) -> anyhow::Result<String> {
    let reply = Body::Cas { msg_id, key: "counter".to_string(), from, to, create_if_not_exists };
    
    let response = serde_json::json!({
        "src": src,
        "dest": dest,
        "body": reply,
    });
    let json = serde_json::to_string(&response)?;

    Ok(json)
}
