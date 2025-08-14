use std::{
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};
use rand::Rng;

const CUSTOM_EPOCH: u64 = 1577836800000; // Jan 1 2020 UTC in ms
const NODE_BITS: u64 = 10;
const SEQ_BITS: u64 = 12;

const MAX_NODE_ID: u64 = (1 << NODE_BITS) - 1;
const MAX_SEQ: u64 = (1 << SEQ_BITS) - 1;

pub struct Snowflake {
    inner: Mutex<SnowflakeState>,
    node_id: u64,
}

struct SnowflakeState {
    last_ts: u64,
    sequence: u64,
}

impl Snowflake {
    /// Creates a new Snowflake with a random node ID.
    pub fn new() -> Self {
        let node_id = rand::rng().random_range(0..=MAX_NODE_ID);
        Self {
            inner: Mutex::new(SnowflakeState { last_ts: 0, sequence: 0 }),
            node_id,
        }
    }

    pub fn next_id(&self) -> u64 {
        let mut state = self.inner.lock().unwrap();

        let mut ts = current_time_ms();

        if ts < state.last_ts {
            ts = wait_next_ms(state.last_ts);
        }

        if ts == state.last_ts {
            state.sequence += 1;
            if state.sequence > MAX_SEQ {
                ts = wait_next_ms(state.last_ts);
                state.sequence = 0;
                state.last_ts = ts;
            }
        } else {
            state.sequence = 0;
            state.last_ts = ts;
        }

        ((ts - CUSTOM_EPOCH) << (NODE_BITS + SEQ_BITS))
            | ((self.node_id & MAX_NODE_ID) << SEQ_BITS)
            | (state.sequence & MAX_SEQ)
    }
}

fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

fn wait_next_ms(last_ts: u64) -> u64 {
    let mut ts = current_time_ms();
    while ts <= last_ts {
        ts = current_time_ms();
    }
    ts
}