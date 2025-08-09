
use maelstrom_rust_node::{message::{Body, Message}, process_message_line, state::State};

/// Helper to create a standard Init message
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

pub fn make_generate_msg(msg_id: u64) -> Message {
    Message {
        src: "client".to_string(),
        dest: "node1".to_string(),
        body: Body::Generate { msg_id },
    }
}


/// Serialize a Message struct into a JSON string
pub fn to_json_string(msg: &Message) -> String {
    serde_json::to_string(msg).expect("Failed to serialize message")
}

/// Deserialize JSON output string back into a Message struct

pub fn parse_reply(output: &str) -> serde_json::Value {
    serde_json::from_str(output).expect("Failed to parse reply JSON")
}


/// Runs the provided message JSON through your process_message_line function,
/// captures and returns the output as a String
pub fn run_test_message(input_msg: &Message, state: &mut State) -> String {
    let input_json = to_json_string(input_msg);
    let mut output = Vec::new();

    process_message_line(input_json, state, &mut output).expect("Processing message failed");

    String::from_utf8(output).expect("Output is not valid UTF-8")
}
