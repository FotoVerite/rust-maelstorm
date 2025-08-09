use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

pub struct Storage {
    pub topology: HashSet<String>,
    values: BTreeMap<u64, u64>,
    peer_pending: BTreeMap<String, BTreeSet<u64>>,
    pub counter: AtomicU64, // ...
}

impl Storage {
    pub fn new() -> Self {
        Self {
            topology: HashSet::new(),
            values: BTreeMap::new(),
            peer_pending: BTreeMap::new(),
            counter: AtomicU64::new(0),
        }
    }

    pub fn values(&self) -> Vec<u64> {
        self.values.values().map(|v| v.clone()).collect::<Vec<u64>>()
    }

    pub fn update_typology(&mut self, update: Vec<String>) {
        for node in update {
            if self.topology.insert(node.clone()) {
                self.peer_pending
                    .insert(node.clone(), self.values.keys().cloned().collect());
            }
        }
    }

    pub fn update_values(&mut self, message: u64) {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64;
        let seq = self.counter.fetch_add(1, Ordering::Relaxed); // 12 bits
        let key = pack_key(ts, seq);
        self.values.insert(key, message);
        let nodes: Vec<String> = self.topology.iter().cloned().collect();
        for node in nodes {
            self.add_to_pending(node.clone(), key);
        }
    }

    fn add_to_pending(&mut self, node: String, key: u64) {
        let entry = self.peer_pending.entry(node).or_insert(BTreeSet::new());
        entry.insert(key);
    }
}

fn pack_key(ts: u64, seq: u64) -> u64 {
    (ts << 22) | (seq & 0x3F_FFFF) // 0x3F_FFFF masks 22 bits
}

fn _unpack_key(key: u64) -> (u64, u64) {
    let ts = key >> 22;
    let seq = key & 0x3F_FFFF;
    (ts, seq)
}