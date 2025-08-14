use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use tokio::{
    sync::mpsc::{Receiver},
    time::interval,
};

use crate::{
    broadcast::actor::BroadcastCommand,
    message::BroadcastMessage,
    storage::{SharedStore, node_context::NodeContext},
};

pub struct BroadcastWorker {
    ctx: NodeContext,
    store: SharedStore,
    key_map: HashMap<u64, u64>,             // Arc<Mutex<ValueStore>>
    sent_to: HashMap<String, HashSet<u64>>, // delta tracking
    queue: HashMap<String, HashSet<u64>>,
    rx: Receiver<(String, u64)>,
}

impl BroadcastWorker {
    pub fn new(ctx: NodeContext, store: SharedStore, rx: Receiver<(String, u64)>) -> Self {
        Self {
            ctx,
            store,
            key_map: HashMap::new(),
            queue: HashMap::new(),
            sent_to: HashMap::new(),
            rx,
        }
    }

    pub async fn run(&mut self) {
        let mut ticker = interval(Duration::from_millis(250));
        //let mut gossip_interval = interval(Duration::from_millis(1000));

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    let neighbors = self.ctx.online_neighbors();
                    for node in neighbors {
                        let received_by_node = self.sent_to.entry(node.clone()).or_insert(HashSet::new());
                        let store = self.store.read().unwrap();
                        let queue = self.queue.entry(node.clone()).or_insert(HashSet::new());
                        for value in store.iter() {
                            if received_by_node.contains(value) {
                                continue
                            }
                            queue.insert(value.clone());
                        }

                    };

                    for (dest, node_queue) in self.queue.iter_mut() {
                        for val in node_queue.iter() {
                            let key = self.ctx.next_id();
                            self.key_map.insert(key, *val);
                             self.ctx.broadcast(BroadcastCommand::Broadcast {
                                dest: dest.clone(),
                                msg_id: key.clone(),
                                message: BroadcastMessage::Single(*val),
                            })
                        }
                    }
                }
                Some((node, msg_id)) = self.rx.recv() => {
                    if let Some(value) = self.key_map.remove(&msg_id) {
                            self.sent_to.entry(node.clone())
                            .or_default()
                            .insert(value);
                    if let Some(queue) = self.queue.get_mut(&node) {
                            queue.retain(|v| *v != value);
                        }
                    }
                },

            }
        }
    }
}
