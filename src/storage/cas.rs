use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PendingRequest {
    pub dest: String,
    pub from: u64,
    pub msg_id: u64,
    pub to: u64,
}

use crate::broadcast::actor::BroadcastCommand;

use super::Storage;

impl Storage {
    pub async fn process_add(&mut self, delta: u64, dest: String, msg_id: u64) {
        let key = self.next_id();
        let from = self.g_counter_value();
        let to = from + delta;
        let mut update = HashMap::new();
        update.insert(self._node_id.as_ref().unwrap().clone(), to);
        self.update_counter(update);

        self.pending_cas.insert(
            key,
            PendingRequest {
                dest,
                from,
                to,
                msg_id,
            },
        );
        let _ = self.tx
            .send(BroadcastCommand::Cas {
                dest: "seq-kv".to_string(),
                msg_id: key,
                from,
                to,
                create_if_not_exists: true,
            })
            .await;
    }

    pub async fn retry_for_cas(&mut self, msg_id: u64) {
        if let Some(mut cas_request) = self.pending_cas.remove(&msg_id) {
            let key = self.next_id();
            if self.g_counter_value() > cas_request.to {
                return;
            }
            cas_request.from = self.g_counter_value();
            self.pending_cas.insert(key, cas_request.clone());
            let _ = self.tx
                .send(BroadcastCommand::Cas {
                    dest: "seq-kv".to_string(),
                    msg_id: key,
                    from: cas_request.from,
                    to: cas_request.to,
                    create_if_not_exists: true,
                })
                .await;
        }
    }

    pub fn remove_request_from_pending_cas(&mut self, msg_id: u64) {
        eprintln!("{:?} {:?}", self.pending_cas, msg_id);

        if let Some(cas_request) = self.pending_cas.remove(&msg_id) {
            let mut update = HashMap::new();
            update.insert(self._node_id.as_ref().unwrap().clone(), cas_request.to);
            self.update_counter(update);
        } else {
            panic!("There was no key found")
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::storage::Storage;
    use super::*;

    #[tokio::test]
    async fn process_add_sends_cas_and_updates_pending() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(10);
        let mut store = Storage::new(tx);
        store.set_id("node-A").await;

        store.process_add(10, "node-B".to_string(), 1).await;

        assert_eq!(store.pending_cas.len(), 1);

    }

    #[tokio::test]
    async fn retry_for_cas_sends_cas_and_updates_pending() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(10);
        let mut store = Storage::new(tx);
        store.set_id("node-A").await;

        store.pending_cas.insert(1, PendingRequest { dest: "node-B".to_string(), from: 0, to: 10, msg_id: 1 });

        store.retry_for_cas(1).await;

        assert_eq!(store.pending_cas.len(), 1);
        let broadcast = rx.recv().await.unwrap();
        assert!(matches!(broadcast, BroadcastCommand::Cas { .. }));
    }

    #[tokio::test]
    async fn remove_request_from_pending_cas_removes_and_updates_counter() {
        let (tx, _rx) = tokio::sync::mpsc::channel(10);
        let mut store = Storage::new(tx);
        store.set_id("node-A").await;

        store.pending_cas.insert(1, PendingRequest { dest: "node-B".to_string(), from: 0, to: 10, msg_id: 1 });

        store.remove_request_from_pending_cas(1);

        assert_eq!(store.pending_cas.len(), 0);
        assert_eq!(store.g_counter_value(), 10);
    }
}