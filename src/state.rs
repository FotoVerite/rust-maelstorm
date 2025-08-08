pub struct State {
    pub node_id: Option<String>,
    // other fields per challenge, e.g.:
    pub echo_log: Vec<String>,
    pub broadcast_peers: Vec<String>,
    pub current_workload: Option<String>
    // ...
}

impl State {
    pub fn new() -> Self {
        Self {
            node_id: None,
            current_workload: None,
            echo_log: vec![],
            broadcast_peers: vec![],
        }
    }
}
