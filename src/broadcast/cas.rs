use tokio::sync::mpsc::Sender;

use crate::{
    message::{Body},
};

pub async fn send_cas (
    src: String,
    dest: String,
    msg_id: u64,
    from: u64, 
    to: u64,
    create_if_not_exists: bool,
    tx: Sender<String>,
) -> anyhow::Result<()> {
    let reply = Body::Cas { msg_id, key: "counter".to_string(), from, to, create_if_not_exists };
    
    let response = serde_json::json!({
        "src": src,
        "dest": dest,
        "body": reply,
    });
    let json = serde_json::to_string(&response)?;

    Ok(tx.send(json).await?)
}
