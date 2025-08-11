use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, mpsc::Sender};

use crate::{broadcast::actor::BroadcastCommand, message::BroadcastMessage, storage::node_state::NodeStatus};

use super::Storage;

impl Storage {
    pub fn update_values(&mut self, src: String, message: u64) {
        let id = self.node_id_to_u64();
        let key = self.snowflake.next_id(id);
        self.values.insert(key, (src.to_string(), message));
        let nodes: Vec<String> = self.topology.iter().cloned().collect();
        for node in nodes {
            if node != src {
                self.add_to_pending(node.clone(), key);
            }
        }
    }

    pub fn remove_from_peer_pending(&mut self, node: String, key: u64) {
        let now: u64 = (self.clock)();
        if let Some(candidates) = self.peer_pending.get_mut(&node) {
            candidates.remove(&key);
        }
        if let Some(status) = self.node_status.get_mut(&node) {
            match status {
                NodeStatus::Online(time) => *time = now,
                NodeStatus::Rejoining(_, _) => *status = NodeStatus::Online(now),
                NodeStatus::Offline(_last_seen) => {
                    *status = NodeStatus::Online(now);
                    let known = &self.values;
                    let pending = self
                        .peer_pending
                        .entry(node.clone())
                        .or_insert(BTreeSet::new());
                    for k in known.keys() {
                        pending.insert(*k);
                    }
                }
            };
        }
    }

    fn add_to_pending(&mut self, node: String, key: u64) {
        let entry = self.peer_pending.entry(node).or_insert(BTreeSet::new());
        entry.insert(key);
    }
}

pub async fn spawn_gossip_sender(arc_storage: Arc<Mutex<Storage>>, tx: Sender<BroadcastCommand>) {
    tokio::spawn(async move {
        let mut online_interval = tokio::time::interval(Duration::from_secs(1));
        let mut offline_interval = tokio::time::interval(Duration::from_secs(3));
        loop {
            tokio::select! {
                            _ = online_interval.tick() => {

                                let mut to_send = Vec::new();

                                {
                                    let mut storage = arc_storage.lock().await;
                                    if storage._node_id.is_none() {
                                        continue;
                                    }
                                    storage.update_node_states();

                                    for node in storage.online_nodes() {
                                        if let Some(pending) = storage.peer_pending.get(node) {
                                            for key in pending.iter() {
                                                if let Some(message) = storage.values.get(key) {
                                                    to_send.push((node.clone(), *key, message.1));
                                                }
                                            }
                                        }
                                    }
                                } // lock dropped here

                                for (dest, msg_id, message) in to_send {
                                    let _ = tx
                                        .send(BroadcastCommand::Broadcast {
                                            dest,
                                            msg_id,
                                            message: BroadcastMessage::Single(message),
                                        })
                                        .await;
                                }
                            }

                            _ = offline_interval.tick() => {

                                let mut to_send = Vec::new();

                                {
                                    let  storage = arc_storage.lock().await;
                                    if storage._node_id.is_none() {
                                        continue;
                                    }

                                    for node in storage.offline_nodes() {
                                        if let Some(pending) = storage.peer_pending.get(node) {
                                            for key in pending.iter() {
                                                if let Some(message) = storage.values.get(key) {
                                                    to_send.push((node.clone(), *key, message.1));
                                                }
                                            }
                                        }
                                    }
                                } // lock dropped here

                                for (dest, msg_id, message) in to_send {
                                    let _ = tx
                                        .send(BroadcastCommand::Broadcast {
                                            dest,
                                            msg_id,
                                                                            message: BroadcastMessage::Single(message)
            ,
                                        })
                                        .await;
                                }
                            }
                        }
        }
    });
}


#[cfg(test)]
mod tests {
    use crate::storage::Storage;
    use super::*;

    #[tokio::test]
    async fn update_values_adds_message_and_marks_all_peers_pending_except_src() {
        let (tx, _rx) = tokio::sync::mpsc::channel(10);
        let mut store = Storage::new(tx);
        store.set_id("node-A").await;

        store.update_typology(vec!["node-B".into(), "node-C".into()]);

        store.update_values("node-A".to_string(), 42);

        let key = *store.values.keys().next().unwrap();
        let val = store.values.get(&key).unwrap();
        assert_eq!(val.1, 42);

        // node-B and node-C should have the key pending, but node-A should not
        for peer in ["node-B", "node-C"] {
            assert!(store.peer_pending.get(peer).unwrap().contains(&key));
        }
        assert!(
            !store.peer_pending.contains_key("node-A")
                || !store.peer_pending["node-A"].contains(&key)
        );
    }

    #[tokio::test]
    async fn remove_from_peer_pending_removes_key_and_updates_status() {
        let (tx, _rx) = tokio::sync::mpsc::channel(10);
        let mut store = Storage::new(tx);
        store.set_id("node-A").await;

        store.update_typology(vec!["node-B".into()]);
        store.update_values("node-A".to_string(), 123);

        let key = *store.values.keys().next().unwrap();

        store.remove_from_peer_pending("node-B".to_string(), key);

        assert!(store.peer_pending.get("node-B").unwrap().is_empty());
        assert!(matches!(
            store.node_status.get("node-B"),
            Some(NodeStatus::Online(_))
        ));
    }
}