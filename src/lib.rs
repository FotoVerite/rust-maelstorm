use std::io::Write;

use anyhow::Context;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::handlers::broadcast::handle_broadcast;
use crate::handlers::echo::handle_echo;
use crate::handlers::id_gen::handle_id_gen;
use crate::handlers::init::handle_init;
use crate::handlers::read::handle_read;
use crate::handlers::topology::handle_topology;
use crate::message::{Body, Message};
use crate::state::State;
use crate::storage::Storage;

pub mod handlers;
pub mod message;
pub mod state;
pub mod storage;

pub trait Handler {
    /// Handle an incoming message, possibly mutating state, and produce zero or more responses.
    fn handle(&mut self, msg: &Message, state: &mut State) -> Vec<Message>;
}

pub async fn process_message_line(
    line: String,
    state: &mut State,
    storage: &mut Storage,
    mut tx: Sender<String>,
) -> anyhow::Result<()> {
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
            node_ids: _,
            workload: _,
        } => {
            state.node_id = Some(node_id);
            handle_init(src, dest, msg_id, tx).await
        }
        Body::Broadcast { msg_id, message } => {
            handle_broadcast(src, dest, msg_id, storage, message, tx).await
        }
        Body::Echo { msg_id, echo } => handle_echo(src, dest, msg_id, echo, tx).await,
        Body::Generate { msg_id } => handle_id_gen(src, dest, msg_id, state, tx).await,
        Body::Read { msg_id } => handle_read(src, dest, msg_id, storage, tx).await,
        Body::Topology { msg_id, topology } => {
            let node_id = &state
                .node_id
                .as_ref()
                .expect("Node id has not been set")
                .clone();
            let node = topology.get(node_id).expect("should have found node");
            handle_topology(src, dest, msg_id, storage, node.clone(), tx).await
        }
    }
}

pub async fn write_stdout<W: Write>(mut writer: W, mut rx: Receiver<String>) -> anyhow::Result<()> {
    while let Some(msg) = rx.recv().await {
        if let Err(e) = writeln!(writer, "{}", msg).context("Failed to output to stdout") {
            eprintln!(
                "Writer error ({}), shutting down response handler for msg: {:?}",
                e, msg
            );
            break;
        }
    }
    Ok(())
}
