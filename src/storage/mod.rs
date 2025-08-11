pub mod cas;
pub mod g_counter;
pub mod node_state;
pub mod value_store;

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use tokio::sync::{Mutex, mpsc::Sender};

use crate::{broadcast::actor::BroadcastCommand, snowflake::Snowflake};

use self::{cas::PendingRequest, node_state::NodeStatus};

pub type NodeId = Arc<Mutex<Option<String>>>;

pub struct Storage {
    pub node_id: NodeId,
    _node_id: Option<String>,
    pub topology: HashSet<String>,
    pub values: BTreeMap<u64, (String, u64)>,
    pub peer_pending: BTreeMap<String, BTreeSet<u64>>,
    pending_cas: HashMap<u64, PendingRequest>,

    pub snowflake: Snowflake, // ...
    pub node_status: HashMap<String, NodeStatus>,
    pub workload: Option<String>,
    clock: Arc<dyn Fn() -> u64 + Send + Sync>,
    pub counter: HashMap<String, u64>,
    pub tx: Sender<BroadcastCommand>,
}

impl Storage {
    pub fn new_with_clock<F>(tx: Sender<BroadcastCommand>, clock: F) -> Self
    where
        F: Fn() -> u64 + Send + Sync + 'static,
    {
        let storage = Self {
            node_id: Arc::new(Mutex::new(None)),
            _node_id: None,
            topology: HashSet::new(),
            values: BTreeMap::new(),
            peer_pending: BTreeMap::new(),
            pending_cas: HashMap::new(),
            snowflake: Snowflake::new(),
            node_status: HashMap::new(),
            clock: Arc::new(clock),
            counter: HashMap::new(),
            workload: Some("".into()),
            tx,
        };
        storage
    }

    pub fn new(tx: Sender<BroadcastCommand>) -> Self {
        Self::new_with_clock(tx, || time_now())
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
}

fn time_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[allow(dead_code)]
    async fn make_storage_with_id(id: &str) -> Arc<Mutex<Storage>> {
        let (tx, _rx) = tokio::sync::mpsc::channel(10);
        let mut store = Storage::new(tx);
        store.set_id(id).await;
        Arc::new(Mutex::new(store))
    }

    #[tokio::test]
    async fn set_id_stores_node_id() {
        let (tx, _rx) = tokio::sync::mpsc::channel(10);
        let mut store = Storage::new(tx);
        store.set_id("node-A").await;

        assert_eq!(store._node_id.as_deref(), Some("node-A"));
        let node_id_guard = store.node_id.lock().await;
        assert_eq!(node_id_guard.as_deref(), Some("node-A"));
    }

    #[tokio::test]
    async fn update_typology_inserts_nodes_and_marks_online() {
        let (tx, _rx) = tokio::sync::mpsc::channel(10);
        let mut store = Storage::new(tx);
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

    #[tokio::test]
    async fn update_node_states_moves_stale_nodes_offline() {
        let (tx, _rx) = tokio::sync::mpsc::channel(10);
        let mut store = Storage::new(tx);
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