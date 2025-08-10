use maelstrom_rust_node::{message::{Body, Message}};

/// Helper to create a standard Init message
#[allow(dead_code)]
pub fn make_init_msg() -> Message {
    Message {
        src: "client".to_string(),
        dest: "node1".to_string(),
        body: Body::Init {
            msg_id: 1,
            node_id: "node1".to_string(),
            node_ids: vec!["node1".to_string(), "node2".to_string()],
            workload: Some("echo".to_string()),
        },
    }
}

/// Helper to create an Echo message with given echo text
#[allow(dead_code)]
pub fn make_echo_msg(echo_text: &str) -> Message {
    Message {
        src: "client".to_string(),
        dest: "node1".to_string(),
        body: Body::Echo {
            msg_id: 2,
            echo: echo_text.to_string(),
        },
    }
}

#[allow(dead_code)]
pub fn make_generate_msg(msg_id: u64) -> Message {
    Message {
        src: "client".to_string(),
        dest: "node1".to_string(),
        body: Body::Generate { msg_id },
    }
}


/// Serialize a Message struct into a JSON string
#[allow(dead_code)]
pub fn to_json_string(msg: &Message) -> String {
    serde_json::to_string(msg).expect("Failed to serialize message")
}

/// Deserialize JSON output string back into a Message struct
#[allow(dead_code)]
pub fn parse_reply(output: &str) -> serde_json::Value {
    serde_json::from_str(output).expect("Failed to parse reply JSON")
}

#[allow(dead_code)]
pub fn make_broadcast_msg(msg_id: u64, message: u64) -> Message {
    Message {
        src: "client".to_string(),
        dest: "node1".to_string(),
        body: Body::Broadcast {
            msg_id,
            message,
        },
    }
}

#[allow(dead_code)]
pub fn make_read_msg(msg_id: u64) -> Message {
    Message {
        src: "client".to_string(),
        dest: "node1".to_string(),
        body: Body::Read {
            msg_id,
        },
    }
}

#[allow(dead_code)]
pub fn make_topology_msg(msg_id: u64) -> Message {
    let mut topology = std::collections::HashMap::new();
    topology.insert("node1".to_string(), vec!["node2".to_string(), "node3".to_string()]);
    Message {
        src: "client".to_string(),
        dest: "node1".to_string(),
        body: Body::Topology {
            msg_id,
            topology,
        },
    }
}