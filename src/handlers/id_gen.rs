use std::io::Write;

use anyhow::Context;

use crate::{
    message::ReplyBody, state::State,
};

pub fn handle_id_gen<W: Write>(
    src: String,
    dest: String,
    msg_id: u64,
    state: &mut State,
    out: &mut W,
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

    writeln!(out, "{}", json).context("Error sanding Echo Message")
}
