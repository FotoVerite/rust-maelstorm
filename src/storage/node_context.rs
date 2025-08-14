use crate::{broadcast::actor::BroadcastCommand, storage::StorageBroadcastCommand};

pub struct NodeContext {
    pub get_node_id: Box<dyn Fn() -> String + Send + Sync>,
    pub next_id: Box<dyn Fn() -> u64 + Send + Sync>,
    pub online_neighbors: Box<dyn Fn() -> Vec<String> + Send + Sync>,
    pub offline_neighbors: Box<dyn Fn() -> Vec<String> + Send + Sync>,

    pub gossip: Box<dyn Fn(StorageBroadcastCommand) + Send + Sync>,
    pub broadcast: Box<dyn Fn(BroadcastCommand) + Send + Sync>,
}

impl NodeContext {
    pub fn node_id(&self) -> String {
        (self.get_node_id)()
    }

    pub fn next_id(&self) -> u64 {
        (self.next_id)()
    }

    pub fn online_neighbors(&self) -> Vec<String> {
        (self.online_neighbors)()
    }

    pub fn offline_neighbors(&self) -> Vec<String> {
        (self.offline_neighbors)()
    }

    pub fn gossip(&self, command: StorageBroadcastCommand) {
        (self.gossip)(command)
    }

    pub fn broadcast(&self, command: BroadcastCommand) {
        (self.broadcast)(command)
    }
}
