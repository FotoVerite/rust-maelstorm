use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub src: String,
    pub dest: String,
    pub body: Body,
}

#[derive(Serialize, Deserialize)]
pub struct ReplyMessage {
    pub src: String,
    pub dest: String,
    pub body: ReplyBody,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Body {
    #[serde(rename = "init")]
    Init {
        msg_id: u64,
        node_id: String,
        node_ids: Vec<String>,
        workload: Option<String>,
    },
    #[serde(rename = "broadcast")]
    Broadcast { msg_id: u64, message: u64 },
    #[serde(rename = "broadcast_ok")]
    BroadcastOk { in_reply_to: u64 },
    #[serde(rename = "echo")]
    Echo { msg_id: u64, echo: String },
    #[serde(rename = "generate")]
    Generate { msg_id: u64 },
    #[serde(rename = "read")]
    Read { msg_id: u64 },
    #[serde(rename = "topology")]
    Topology {
        msg_id: u64,
        topology: HashMap<String, Vec<String>>,
    }, // Add more variants as needed
}

#[derive(Serialize, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ReplyBody {
    #[serde(rename = "broadcast_ok")]
    BroadcastOk { in_reply_to: u64 },
    #[serde(rename = "init_ok")]
    InitOk { in_reply_to: u64 },
    #[serde(rename = "echo_ok")]
    EchoOk { in_reply_to: u64, echo: String },
    #[serde(rename = "generate_ok")]
    GenerateOk { id: String, in_reply_to: u64 },
    #[serde(rename = "read_ok")]
    ReadOk {
        in_reply_to: u64,
        messages: Vec<u64>,
    },
    #[serde(rename = "topology_ok")]
    TopologyOk { in_reply_to: u64 },
}
