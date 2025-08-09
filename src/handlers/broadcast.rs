use std::io::Write;

use anyhow::Context;

use crate::{
    message::{ReplyBody},
    storage::Storage,
};

pub fn handle_broadcast<W: Write>(
    src: String,
    dest: String,
    msg_id: u64,
    storage: &mut Storage,
    message: u64,
    out: &mut W,
) -> anyhow::Result<()> {
    storage.update_values(message);
    let reply = ReplyBody::BroadcastOk {
        in_reply_to: msg_id,
    };
    let response = serde_json::json!({
        "src": dest,
        "dest": src,
        "body": reply,
    });
    let json = serde_json::to_string(&response)?;

    writeln!(out, "{}", json).context("Error sanding Broadcast Message")
}
