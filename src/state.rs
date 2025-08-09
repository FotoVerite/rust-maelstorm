use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

pub struct State {
    pub node_id: Option<String>,
    // other fields per challenge, e.g.:
    pub echo_log: Vec<String>,
    pub broadcast_peers: Vec<String>,
    pub current_workload: Option<String>,
    pub counter: AtomicU64, // ...
}

impl State {
    pub fn new() -> Self {
        Self {
            node_id: None,
            current_workload: None,
            echo_log: vec![],
            broadcast_peers: vec![],
            counter: AtomicU64::new(0),
        }
    }

    pub fn next_id(&self) -> String {
        match &self.node_id {
            Some(id) => {
                let ts = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_millis() as u64;

                let seq = self.counter.fetch_add(1, Ordering::Relaxed); // 12 bits
                format!("{id}-{ts:013}-{seq:06}")
            }
            None => panic!("Node id has not been sent"),
        }
    }
}
