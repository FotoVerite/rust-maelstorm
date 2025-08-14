use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum BroadcastMessage {
    Single(u64),
    Multiple(Vec<u64>),
    Hashmap(HashMap<String, u64>),
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

    #[serde(rename = "add")]
    Add { msg_id: u64, delta: u64 },
    #[serde(rename = "broadcast")]
    Broadcast {
        msg_id: u64,
        message: BroadcastMessage,
    },
    #[serde(rename = "broadcast_ok")]
    BroadcastOk { in_reply_to: u64 },
    #[serde(rename = "echo")]
    Echo { msg_id: u64, echo: String },
    #[serde(rename = "error")]
    Error {
        in_reply_to: u64,
        code: u64,
        text: String,
    },
    #[serde(rename = "generate")]
    Generate { msg_id: u64 },
    #[serde(rename = "read")]
    Read { msg_id: u64, key: Option<String> },
    #[serde(rename = "cas")]
    Cas {
        msg_id: u64,
        key: String,
        from: u64,
        to: u64,
        create_if_not_exists: bool,
    },
    #[serde(rename = "cas_ok")]
    CasOk { in_reply_to: u64 },
    #[serde(rename = "topology")]
    Topology {
        msg_id: u64,
        topology: HashMap<String, Vec<String>>,
    }, // Add more variants as needed
}

impl Body {
    pub fn cache_id(&self) -> &u64 {
        match self {
            Self::Add { msg_id, .. } => msg_id,
            Self::Broadcast { msg_id, .. } => msg_id,
            Self::BroadcastOk { in_reply_to, .. } => in_reply_to,
            Self::Cas { msg_id, .. } => msg_id,
            Self::CasOk { in_reply_to, .. } => in_reply_to,
            Self::Echo { msg_id, .. } => msg_id,
            Self::Error { in_reply_to, .. } => in_reply_to,
            Self::Generate { msg_id, .. } => msg_id,
            Self::Init { msg_id, .. } => msg_id,
            Self::Read { msg_id, .. } => msg_id,
            Self::Topology { msg_id, .. } => msg_id,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ReadMessage {
    Single(u64),
    Multiple(Vec<u64>),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ReplyBody {
    #[serde(rename = "add_ok")]
    AddOk { in_reply_to: u64 },
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
        messages: ReadMessage,
    },
    #[serde(rename = "topology_ok")]
    TopologyOk { in_reply_to: u64 },
}
