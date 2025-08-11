use std::collections::HashMap;

use crate::{broadcast::actor::BroadcastCommand, message::BroadcastMessage};

use super::Storage;

impl Storage {
    pub fn g_counter_value(&self) -> u64 {
        self.counter.values().sum()
    }

    pub fn g_counter_node_value(&mut self) -> u64 {
        let node_id = self._node_id.as_ref().expect("Node Id not set");
        let value = self.counter.entry(node_id.to_string()).or_insert(0);
        value.clone()
    }

    pub fn update_counter(&mut self, mut values: HashMap<String, u64>) {
        for (node, value) in values.iter_mut() {
            let entry = self.counter.entry(node.clone()).or_insert(value.clone());
            if entry < value {
                *entry = value.clone()
            }
        }
        let nodes: Vec<String> = self.topology.iter().cloned().collect();
        for node in nodes {
            _ = self.tx.send(BroadcastCommand::Broadcast {
                dest: node,
                msg_id: self.next_id(),
                message: BroadcastMessage::Hashmap(self.counter.clone()),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Storage;

    #[tokio::test]
    async fn g_counter_value_returns_sum_of_all_counters() {
        let (tx, _rx) = tokio::sync::mpsc::channel(10);
        let mut store = Storage::new(tx);
        store.set_id("node-A").await;

        store.counter.insert("node-A".to_string(), 10);
        store.counter.insert("node-B".to_string(), 20);

        assert_eq!(store.g_counter_value(), 30);
    }

    #[tokio::test]
    async fn g_counter_node_value_returns_value_for_node() {
        let (tx, _rx) = tokio::sync::mpsc::channel(10);
        let mut store = Storage::new(tx);
        store.set_id("node-A").await;

        store.counter.insert("node-A".to_string(), 10);

        assert_eq!(store.g_counter_node_value(), 10);
    }

    #[tokio::test]
    async fn update_counter_updates_values_and_broadcasts() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(10);
        let mut store = Storage::new(tx);
        store.set_id("node-A").await;
        store.update_typology(vec!["node-B".into()]);

        let mut values = HashMap::new();
        values.insert("node-A".to_string(), 20);
        values.insert("node-B".to_string(), 30);

        store.update_counter(values);

        assert_eq!(store.counter.get("node-A"), Some(&20));
        assert_eq!(store.counter.get("node-B"), Some(&30));
    }
}
