use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

use crate::{message::ReplyBody, storage::Storage};

pub async fn handle_topology(
    src: String,
    dest: String,
    msg_id: u64,
    topology: HashMap<String, Vec<String>>,
    storage: Arc<Mutex<Storage>>,
) -> anyhow::Result<String> {
    let mut guard = storage.lock().await;
    let node = topology.get(&guard.get_node_id()).expect("should have found node");
    guard.update_neighbors(node.clone());
    let reply = ReplyBody::TopologyOk {
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
