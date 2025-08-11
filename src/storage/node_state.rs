

pub enum NodeStatus {
    Online(u64),
    Offline(u64),
    Rejoining(u64, u64),
    // Add others if needed (e.g., Suspected, Failed)
}

impl super::Storage {
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

#[cfg(test)]
mod tests {
    use crate::storage::Storage;
    use super::*;

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
    async fn update_node_states_moves_stale_nodes_offline() {
        let (tx, _rx) = tokio::sync::mpsc::channel(10);
        let mut store = Storage::new(tx);
        store.set_id("node-A").await;

        // Insert a node with old online timestamp
        store
            .node_status
            .insert("node-B".into(), NodeStatus::Online(crate::storage::time_now() - 60_000));

        store.update_node_states();

        assert!(matches!(
            store.node_status.get("node-B"),
            Some(NodeStatus::Offline(_))
        ));
    }
}