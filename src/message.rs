use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub src: String,
    pub dest: String,
    pub body: Body,
}

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
    #[serde(rename = "echo")]
    Echo { msg_id: u64, echo: String },
    // Add more variants as needed
}


#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum ReplyBody {
    #[serde(rename = "init_ok")]
    InitOk { in_reply_to: u64 },

    #[serde(rename = "echo_ok")]
    EchoOk { in_reply_to: u64, echo: String },

    // Add more variants as needed
}
