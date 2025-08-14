pub mod g_counter;
pub mod neighbors;
pub mod node_context;
pub mod value_store;

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
    time::{SystemTime, UNIX_EPOCH},
};

use tokio::sync::{
    mpsc::{self, Sender},
};

use crate::{
    broadcast::actor::BroadcastCommand,
    snowflake::Snowflake,
    storage::{
        g_counter::GCounter,
        neighbors::{offline_nodes, online_nodes, NodeStatus},
        node_context::NodeContext,
        value_store::ValueStore,
    }, workers::broadcast::BroadcastWorker,
};

pub type SharedNodeId = Arc<RwLock<Option<String>>>;
pub type SharedStore = Arc<RwLock<HashSet<u64>>>;
pub type SharedNeighbors = Arc<RwLock<HashMap<String, NodeStatus>>>;

pub enum StorageBroadcastCommand {
    GCounter(HashMap<String, u64>),
}

pub struct Storage {
    pub node_id: SharedNodeId,
    pub neighbors: SharedNeighbors,
    pub values: ValueStore,
    pub g_counter: GCounter,
    pub workload: Option<String>,
    pub processed: HashMap<u64, String>,
    clock: Arc<dyn Fn() -> u64 + Send + Sync>,
    pub snowflake: Arc<Snowflake>,
    pub tx: Sender<BroadcastCommand>,
}

fn get_node_id(node_id: SharedNodeId) -> impl Fn() -> String + Send + Sync + 'static {
    move || {
        node_id
            .read()
            .unwrap()
            .as_ref()
            .expect("node_id is not set")
            .clone()
    }
}

fn online_neighbors(
    neighbors: SharedNeighbors,
) -> impl Fn() -> Vec<String> + Send + Sync + 'static {
    move || {
        let neighbors = Arc::clone(&neighbors);
        online_nodes(neighbors)
    }
}
fn offline_neighbors(
    neighbors: SharedNeighbors,
) -> impl Fn() -> Vec<String> + Send + Sync + 'static {
    move || {
        let neighbors = Arc::clone(&neighbors);
        offline_nodes(neighbors)
    }
}

impl Storage {
     pub fn new_with_clock<F>(tx: Sender<BroadcastCommand>, clock: F) -> Self
    where
        F: Fn() -> u64 + Send + Sync + 'static,
    {
        let (gossip_sender, _gossip_receiver) = mpsc::channel(1024);
        let (broadcast_worker_tx, broadcast_worker_rx) = mpsc::channel(1024);

        let node_id = Arc::new(RwLock::new(None));
        let store: SharedStore = Arc::new(RwLock::new(HashSet::new()));
        let neighbors: SharedNeighbors = Arc::new(RwLock::new(HashMap::new()));
        let snowflake = Arc::new(Snowflake::new());

        let make_next_id = |snowflake: Arc<Snowflake>| -> Box<dyn Fn() -> u64 + Send + Sync> {
            Box::new(move || {
            
                snowflake.next_id()
            })
        };

        let make_broadcast = |tx: Sender<BroadcastCommand>| {
            let tx_clone = tx.clone();
            move |cmd| {
                let tx_inner = tx_clone.clone();
                tokio::spawn(async move { let _ = tx_inner.send(cmd).await; });
            }
        };

        let make_gossip = |tx: Sender<StorageBroadcastCommand>| {
            let tx_clone = tx.clone();
            move |cmd| {
                let tx_inner = tx_clone.clone();
                tokio::spawn(async move { let _ = tx_inner.send(cmd).await; });
            }
        };

        let g_counter_context = NodeContext {
            broadcast: Box::new(make_broadcast(tx.clone())),
            next_id: make_next_id(Arc::clone(&snowflake)),
            online_neighbors: Box::new(online_neighbors(Arc::clone(&neighbors))),
            offline_neighbors: Box::new(offline_neighbors(Arc::clone(&neighbors))),
            get_node_id: Box::new(get_node_id(Arc::clone(&node_id))),
            gossip: Box::new(make_gossip(gossip_sender.clone())),
        };

        let value_context = NodeContext {
            broadcast: Box::new(make_broadcast(tx.clone())),
            next_id: make_next_id(Arc::clone(&snowflake)),
            online_neighbors: Box::new(online_neighbors(Arc::clone(&neighbors))),
            offline_neighbors: Box::new(offline_neighbors(Arc::clone(&neighbors))),
            get_node_id: Box::new(get_node_id(Arc::clone(&node_id))),
            gossip: Box::new(make_gossip(gossip_sender.clone())),
        };

        let mut broadcast_worker = BroadcastWorker::new(value_context, Arc::clone(&store), broadcast_worker_rx);
        tokio::spawn( async move {broadcast_worker.run().await});

        Self {
            node_id,
            neighbors,
            g_counter: GCounter::new(g_counter_context),
            values: ValueStore::new(store, broadcast_worker_tx),
            clock: Arc::new(clock),
            workload: Some("".into()),
            processed: HashMap::new(),
            snowflake,
            tx,
        }
    }

    pub fn new(tx: Sender<BroadcastCommand>) -> Self {
        Self::new_with_clock(tx, || time_now())
    }

    pub fn get_node_id(&mut self) -> String {
        self.node_id
            .read()
            .unwrap()
            .as_ref()
            .expect("node_id is not set")
            .clone()
    }

    pub fn boxed_next_id(&self) -> Box<dyn Fn() -> u64 + Send + Sync + 'static> {
        let snowflake = Arc::clone(&self.snowflake);
        Box::new(move || snowflake.next_id())
    }


    fn time_now(&self) -> u64 {
        (self.clock)()
    }

    pub async fn set_id(&mut self, id: &str) {
        *self.node_id.write().unwrap() = Some(id.to_string());
    }
}

fn time_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u64
}

#[cfg(test)]
mod tests;
