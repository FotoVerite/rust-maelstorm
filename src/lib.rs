use std::io::Write;

use anyhow::bail;

use crate::handlers::echo::handle_echo;
use crate::handlers::id_gen::handle_id_gen;
use crate::handlers::init::handle_init;
use crate::message::{Body, Message};
use crate::state::State;

pub mod handlers;
pub mod message;
pub mod state;

pub trait Handler {
    /// Handle an incoming message, possibly mutating state, and produce zero or more responses.
    fn handle(&mut self, msg: &Message, state: &mut State) -> Vec<Message>;
}

pub fn process_message_line<W: Write>(line: String, state: &mut State, mut out: W) -> anyhow::Result<()> {
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
            state.node_id = Some(node_id);
            handle_init(src, dest, msg_id, &mut out)
        }
        Body::Echo { msg_id, echo } => handle_echo(src, dest, msg_id, echo, &mut out),
        Body::Generate {msg_id} => handle_id_gen(src, dest, msg_id, state, &mut out),
        _ => {
            bail!("Unhandled Message")
        }
    }
}
