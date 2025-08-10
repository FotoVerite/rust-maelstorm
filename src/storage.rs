use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use tokio::sync::{Mutex, mpsc::Sender};

use crate::{broadcast::actor::BroadcastCommand, snowflake::Snowflake};

pub type NodeId = Arc<Mutex<Option<String>>>;

pub enum NodeStatus {
    Online(u64),
    Offline(u64),
    Rejoining(u64, u64),
    // Add others if needed (e.g., Suspected, Failed)
}

pub struct Storage {
    pub node_id: NodeId,
    _node_id: Option<String>,
    pub topology: HashSet<String>,
    pub values: BTreeMap<u64, (String, u64)>,
    pub peer_pending: BTreeMap<String, BTreeSet<u64>>,
    pub snowflake: Snowflake, // ...
    pub node_status: HashMap<String, NodeStatus>,
    clock: Arc<dyn Fn() -> u64 + Send + Sync>, // <- injected clock
}

impl Storage {
    pub fn new_with_clock<F>(clock: F) -> Self
    where
        F: Fn() -> u64 + Send + Sync + 'static,
    {
        let storage = Self {
            node_id: Arc::new(Mutex::new(None)),
            _node_id: None,
            topology: HashSet::new(),
            values: BTreeMap::new(),
            peer_pending: BTreeMap::new(),
            snowflake: Snowflake::new(),
            node_status: HashMap::new(),
            clock: Arc::new(clock),
        };
        storage
    }

    pub fn new() -> Self {
        Self::new_with_clock(|| time_now())
    }

    pub async fn set_id(&mut self, id: &str) {
        self._node_id = Some(id.to_string());
        *self.node_id.lock().await = Some(id.to_string());
    }

    fn node_id_to_u64(&self) -> u64 {
        let node_id = self._node_id.as_ref().expect("Node Id not set");
        let mut hasher = DefaultHasher::new();
        node_id.hash(&mut hasher);
        hasher.finish()
    }

    pub fn next_id(&self) -> u64 {
        let id = self.node_id_to_u64();
        self.snowflake.next_id(id)
    }

    pub fn values(&self) -> Vec<u64> {
        self.values
            .values()
            .map(|(_, v)| v.clone())
            .collect::<Vec<u64>>()
    }

    pub fn update_typology(&mut self, update: Vec<String>) {
        let to_update = update.clone();
        for node in update {
            if self.topology.insert(node.clone()) {
                self.node_status
                    .insert(node.clone(), NodeStatus::Online((self.clock)()));
                self.peer_pending
                    .insert(node.clone(), self.values.keys().cloned().collect());
            }
        }

        // Optionally remove nodes not in update:
        let to_remove: Vec<_> = self
            .topology
            .iter()
            .filter(|n| !to_update.contains(n))
            .cloned()
            .collect();
        for node in to_remove {
            self.topology.remove(&node);
            self.node_status.remove(&node);
            self.peer_pending.remove(&node);
        }
    }

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

    pub fn online_nodes(&self) -> impl Iterator<Item = &String> {
        self.node_status.iter().filter_map(|(name, status)| {
            if matches!(status, NodeStatus::Online(_)) {
                Some(name)
            } else {
                None
            }
        })
    }

    pub fn offline_nodes(&self) -> impl Iterator<Item = &String> {
        self.node_status.iter().filter_map(|(name, status)| {
            if matches!(status, NodeStatus::Offline(_)) {
                Some(name)
            } else {
                None
            }
        })
    }

    pub fn update_node_states(&mut self) {
        let now = (self.clock)();
        for (_name, status) in self.node_status.iter_mut() {
            if let NodeStatus::Online(time) = status {
                if now > *time + 30_000 {
                    *status = NodeStatus::Offline(now);
                }
            }
        }
    }
}

fn time_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u64
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
                                message,
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
                                message,
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
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

        #[allow(dead_code)]
    async fn make_storage_with_id(id: &str) -> Arc<Mutex<Storage>> {
        let mut store = Storage::new();
        store.set_id(id).await;
        Arc::new(Mutex::new(store))
    }

    #[tokio::test]
    async fn set_id_stores_node_id() {
        let mut store = Storage::new();
        store.set_id("node-A").await;

        assert_eq!(store._node_id.as_deref(), Some("node-A"));
        let node_id_guard = store.node_id.lock().await;
        assert_eq!(node_id_guard.as_deref(), Some("node-A"));
    }

    #[tokio::test]
    async fn update_typology_inserts_nodes_and_marks_online() {
        let mut store = Storage::new();
        store.set_id("node-A").await;

        store.update_typology(vec!["node-B".into(), "node-C".into()]);

        assert!(store.topology.contains("node-B"));
        assert!(store.topology.contains("node-C"));

        assert!(matches!(
            store.node_status.get("node-B"),
            Some(NodeStatus::Online(_))
        ));
        assert!(store.peer_pending.contains_key("node-B"));
    }

    #[tokio::test]
    async fn update_values_adds_message_and_marks_all_peers_pending_except_src() {
        let mut store = Storage::new();
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
        let mut store = Storage::new();
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

    #[tokio::test]
    async fn update_node_states_moves_stale_nodes_offline() {
        let mut store = Storage::new();
        store.set_id("node-A").await;

        // Insert a node with old online timestamp
        store
            .node_status
            .insert("node-B".into(), NodeStatus::Online(time_now() - 60_000));

        store.update_node_states();

        assert!(matches!(
            store.node_status.get("node-B"),
            Some(NodeStatus::Offline(_))
        ));
    }
}
