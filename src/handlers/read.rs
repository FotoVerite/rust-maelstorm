use std::io::Write;

use anyhow::Context;

use crate::{message::ReplyBody, storage::Storage};

pub fn handle_read<W: Write>(
    src: String,
    dest: String,
    msg_id: u64,
    storage: &Storage,
    out: &mut W,
) -> anyhow::Result<()> {
    let reply = ReplyBody::ReadOk {
        in_reply_to: msg_id,
        messages: storage.values(),
    };
    let response = serde_json::json!({
        "src": dest,
        "dest": src,
        "body": reply,
    });
    let json = serde_json::to_string(&response)?;

    writeln!(out, "{}", json).context("Error sanding Read Message")
}
