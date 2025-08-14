use std::collections::HashSet;
use tokio::sync::mpsc::Sender;

use crate::storage::SharedStore;

pub struct ValueStore {
    data: SharedStore,
    seen_values: HashSet<u64>,
    to_worker: Sender<(String, u64)>,
}

impl ValueStore {
    pub fn new(data: SharedStore, tx: Sender<(String, u64)>) -> Self {
        Self {
            data,
            seen_values: HashSet::new(),
            to_worker: tx,
        }
    }

    /// Return all values in store
    pub fn values(&self) -> Vec<u64> {
        self.data.read().unwrap().iter().copied().collect()
    }

    /// Iterator over values starting after a given key
    // pub fn values_from(&self, starting_after: u64) -> impl Iterator<Item = (&u64, &u64)> {
    //     self.data
    //         .range((Excluded(&starting_after), Unbounded))
    //         .into_iter()
    // }

    // /// Send catchup messages in batches
    // pub async fn send_catchup(&self, node: String, starting_after: u64, batch_size: usize) {
    //     let mut batch = Vec::with_capacity(batch_size);

    //     for (key, value) in self.values_from(starting_after) {
    //         let cmd = PendingCommand::Add {
    //             key: *key,
    //             dest: node.clone(),
    //             value: *value,
    //         };
    //         batch.push(cmd);
    //         if batch.len() == batch_size {
    //             if self
    //                 .to_worker
    //                 .send(PendingMessage::Catchup(batch))
    //                 .await
    //                 .is_err()
    //             {
    //                 return; // receiver dropped
    //             }
    //             batch = Vec::with_capacity(batch_size);
    //         }
    //     }

    //     if !batch.is_empty() {
    //         let _ = self.to_worker.send(PendingMessage::Catchup(batch)).await;
    //     }
    // }

    /// Insert a new value if not already present
    /// Returns true if the value was new
    pub fn update_store_data(&mut self, value: u64) -> bool {
        // Insert only if the value itself is new
        if self.seen_values.contains(&value) {
            return false;
        }

        self.data.write().unwrap().insert(value);
        self.seen_values.insert(value);
        true
    }

    /// Remove pending message for a specific key/dest
    pub async fn remove_pending(&self, dest: String, value: u64) {
        let _ = self.to_worker.send((dest, value)).await;
    }

    pub fn contains_value(&self, value: u64) -> bool {
        self.seen_values.contains(&value)
    }
}
