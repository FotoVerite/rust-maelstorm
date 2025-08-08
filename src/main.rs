use std::io::{self, BufRead};

use anyhow::Ok;
use maelstrom_rust_node::{
    handlers::{self, echo::handle_echo, init::handle_init},
    message::{Body, Message},
    state::State,
};

fn main() {
    let mut state = State::new();
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        let line = line.expect("Failed to read line");
        let msg: Message = serde_json::from_str(&line).expect("Invalid JSON");

        // Always handle 'init' globally

        let src = msg.src;
        let dest = msg.dest;
        let body = msg.body;
        // Dispatch to specific handlers based on workload
        match body {
            Body::Init {
                msg_id,
                node_id,
                node_ids,
                workload,
            } => {
                handle_init(src, dest, msg_id, &mut out);
            }
            Body::Echo { msg_id, echo } => {
                handle_echo(src, dest, msg_id, echo, &mut out);
            }
            _ => { /* handle unknown */ }
        }
    }
}
