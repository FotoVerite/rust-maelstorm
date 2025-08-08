use std::io::Write;

use anyhow::Context;

use crate::{
    message::{Message, ReplyBody},
    state::State,
};

pub fn handle_init<W: Write>(
    src: String,
    dest: String,
    msg_id: u64,
    out: &mut W,
) -> anyhow::Result<()> {
    let reply = ReplyBody::InitOk {
        in_reply_to: msg_id,
    };


    let response = serde_json::json!({
        "src": dest,
        "dest": src,
        "body": reply,
    });
    let json = serde_json::to_string(&response)?;

    writeln!(out, "{}", json).context("Error sanding Init Message")
}
