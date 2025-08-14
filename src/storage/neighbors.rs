use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::storage::{Storage};

type SharedNeighbors = Arc<RwLock<HashMap<String, NodeStatus>>>;

#[derive(Debug, Clone)]
pub enum NodeStatus {
    Online(u64),
    Offline(u64),
    Rejoining(u64, u64),
}

impl Storage {
    pub fn update_neighbors(&mut self, update: Vec<String>) {
        let mut neighbors = self.neighbors.write().unwrap();
        let time = self.time_now();

        for node in update {
            match neighbors.get(&node).cloned() {
                None => {
                    neighbors.insert(node.clone(), NodeStatus::Online(time));
                    //let _ = self.values.send_catchup(node, time, 512);
                    
                }
                Some(status) => match status {
                    NodeStatus::Offline(last_seen) => {
                        neighbors.insert(node.clone(), NodeStatus::Rejoining(last_seen, time));
                       //let _ = self.values.send_catchup(node, time, 512);

                    }
                    NodeStatus::Online(_) | NodeStatus::Rejoining(_, _) => {
                        neighbors.insert(node.clone(), NodeStatus::Online(time));

                    }
                },
            }
        }
    }
}

pub fn online_nodes(neighbors: SharedNeighbors) -> Vec<String> {
    let neighbors = neighbors.read().unwrap();
    neighbors
        .iter()
        .filter_map(|(name, status)| {
            if let NodeStatus::Online(_) = status {
                Some(name.clone())
            } else {
                None
            }
        })
        .collect()
}

pub fn offline_nodes(neighbors: SharedNeighbors) -> Vec<String> {
    let neighbors = neighbors.read().unwrap();
    neighbors
        .iter()
        .filter_map(|(name, status)| {
            if let NodeStatus::Offline(_) = status {
                Some(name.clone())
            } else {
                None
            }
        })
        .collect()
}
