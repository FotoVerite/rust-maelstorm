use std::{collections::HashMap, time::Duration};

use anyhow::{Context, Ok};
use tokio::{
    sync::oneshot::{self, Receiver, Sender},
    time::timeout,
};

use crate::{
    broadcast::actor::BroadcastCommand,
    storage::{StorageBroadcastCommand, node_context::NodeContext},
};

pub struct GCounter {
    ctx: NodeContext,
    data: HashMap<String, u64>,
    pending: HashMap<u64, BroadcastCommand>,
    watchers: HashMap<u64, Sender<u64>>,
}

impl GCounter {
    pub fn new(ctx: NodeContext) -> Self {
        Self {
            ctx,
            data: HashMap::new(),
            pending: HashMap::new(),
            watchers: HashMap::new(),
        }
    }

    pub fn local_sum(&self) -> u64 {
        self.data.values().sum()
    }

    pub fn local_value(&mut self) -> u64 {
        let id = self.ctx.node_id();
        let value = self.data.entry(id.to_string()).or_insert(0);
        value.clone()
    }

    pub async fn process_add(&mut self, delta: u64) -> anyhow::Result<()> {
        let node_id = self.ctx.node_id();

        let from = timeout(Duration::from_secs(5), self.set_watcher())
            .await
            .context("CAS failed: watcher dropped before value received")??;
        let to = from + delta;
        let mut update = HashMap::new();
        update.insert(node_id.to_string(), to);
        self.update_counter(update);
        let send_key = self.ctx.next_id();
        let command = BroadcastCommand::Cas {
            dest: "seq-kv".to_string(),
            msg_id: send_key,
            from,
            to,
            create_if_not_exists: true,
        };
        self.send_add(send_key, command);
        Ok(())
    }

    fn set_watcher(&mut self) -> Receiver<u64> {
        let (tx, rx) = oneshot::channel();
        let key = self.ctx.next_id();
        self.watchers.insert(key, tx);
        self.ctx.broadcast(BroadcastCommand::Read {
            dest: "seq-kv".to_string(),
            msg_id: key,
        });
        rx
    }

    pub fn send_add(&mut self, key: u64, command: BroadcastCommand) {
        self.pending.insert(key, command.clone());
        self.ctx.broadcast(command)
    }

    pub async fn retry_for_cas(&mut self, msg_id: u64) -> anyhow::Result<()> {
        if let Some(command) = self.pending.remove(&msg_id) {
            match command {
                BroadcastCommand::Cas {
                    dest,
                    msg_id,
                    from: _,
                    to,
                    create_if_not_exists,
                } => {
                    let fresh = timeout(Duration::from_secs(5), self.set_watcher())
                        .await
                        .context("CAS failed: watcher dropped before value received")??;
                    if fresh > to {
                        return Ok(());
                    }
                    let key = self.ctx.next_id();
                    let new_command = BroadcastCommand::Cas {
                        dest,
                        msg_id,
                        from: fresh,
                        to,
                        create_if_not_exists,
                    };
                    self.pending.insert(key, new_command.clone());
                    self.ctx.broadcast(new_command);
                    return Ok(());
                }
                _ => return Ok(()),
            };
        }
        return Ok(());
    }

    pub fn remove_from_pending(&mut self, msg_id: u64) {
        self.pending.remove(&msg_id);
    }

    pub fn update_counter(&mut self, mut values: HashMap<String, u64>) {
        for (node, value) in values.iter_mut() {
            let entry = self.data.entry(node.clone()).or_insert(value.clone());
            if entry < value {
                *entry = value.clone()
            }
        }
        _ = self
            .ctx
            .gossip(StorageBroadcastCommand::GCounter(self.data.clone()));
    }
}

