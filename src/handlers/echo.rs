use std::io::Write;

use anyhow::Context;

use crate::{
    message::{Message, ReplyBody},
};

pub fn handle_echo<W: Write>(
    src: String,
    dest: String,
    msg_id: u64,
    echo: String,
    out: &mut W,
) -> anyhow::Result<()> {
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

    writeln!(out, "{}", json).context("Error sanding Echo Message")
}
