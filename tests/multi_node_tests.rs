mod test_harness;
mod test_utils;

use test_harness::*;
use test_utils::*;
use maelstrom_rust_node::{message::{Body, Message}, handlers::broadcast::handle_broadcast, handlers::broadcast_ok::handle_broadcast_ok, broadcast::broadcast::send_broadcast, storage::Storage};
use tokio::sync::mpsc;

#[tokio::test]
async fn test_init_and_echo() {
    let mut network = TestNetwork::new();
    network.add_node("node1".to_string(), "echo".to_string()).await;

    let init_msg = make_init_msg();
    network.send_message(init_msg);
    while network.tick().await {}

    let echo_msg = make_echo_msg("hello");
    network.send_message(echo_msg);
    while network.tick().await {}

    // Assert that node1's storage has the correct node_id set
    let node1_storage_arc = network.get_node_storage("node1");
    let node1_storage = node1_storage_arc.lock().await;
    assert_eq!(*node1_storage.node_id.lock().await, Some("node1".to_string()));
}

#[tokio::test]
async fn test_id_generation() {
    let mut network = TestNetwork::new();
    network.add_node("node1".to_string(), "unique-ids".to_string()).await;
    network.add_node("node2".to_string(), "unique-ids".to_string()).await;

    // Init nodes to set their node_ids
    let init_msg1 = Message {
        src: "client".to_string(),
        dest: "node1".to_string(),
        body: Body::Init {
            msg_id: 1,
            node_id: "node1".to_string(),
            node_ids: vec!["node1".to_string(), "node2".to_string()],
            workload: Some("unique-ids".to_string()),
        },
    };
    network.send_message(init_msg1);

    let init_msg2 = Message {
        src: "client".to_string(),
        dest: "node2".to_string(),
        body: Body::Init {
            msg_id: 2,
            node_id: "node2".to_string(),
            node_ids: vec!["node1".to_string(), "node2".to_string()],
            workload: Some("unique-ids".to_string()),
        },
    };
    network.send_message(init_msg2);
    while network.tick().await {}

    // Consume init_ok replies
    let _ = network.get_last_reply().expect("Should have received init_ok for node1");
    let _ = network.get_last_reply().expect("Should have received init_ok for node2");


    let mut generated_ids = std::collections::HashSet::new();

    for i in 0..100 {
        let gen_msg1 = make_generate_msg(i);
        network.send_message(gen_msg1);
        while network.tick().await {}

        let gen_msg2 = make_generate_msg(i + 100);
        network.send_message(gen_msg2);
        while network.tick().await {}

        // Extract the generated ID from the last reply
        if let Some(reply_msg) = network.get_last_reply() {
            let id = reply_msg.body.get("id").unwrap().as_str().unwrap().to_string();
            assert!(generated_ids.insert(id.clone()), "Duplicate ID generated: {}", id);
        } else {
            panic!("Did not receive a reply for generate message 1");
        }

        if let Some(reply_msg) = network.get_last_reply() {
            let id = reply_msg.body.get("id").unwrap().as_str().unwrap().to_string();
            assert!(generated_ids.insert(id.clone()), "Duplicate ID generated: {}", id);
        } else {
            panic!("Did not receive a reply for generate message 2");
        }
    }
    assert_eq!(generated_ids.len(), 200);
}

#[tokio::test]
async fn test_read_messages() {
    let mut network = TestNetwork::new();
    network.add_node("node1".to_string(), "broadcast".to_string()).await;

    // Init node1
    let init_msg = make_init_msg();
    network.send_message(init_msg);
    while network.tick().await {}
    // Consume the init_ok reply
    let _ = network.get_last_reply().expect("Should have received init_ok");

    // Broadcast some messages
    network.send_message(make_broadcast_msg(1, 100));
    network.send_message(make_broadcast_msg(2, 200));
    while network.tick().await {}
    // Consume broadcast_ok replies
    let _ = network.get_last_reply().expect("Should have received broadcast_ok for msg 1");
    let _ = network.get_last_reply().expect("Should have received broadcast_ok for msg 2");


    // Read messages from node1
    let read_msg = make_read_msg(3);
    network.send_message(read_msg);
    while network.tick().await {}

    // Assert that the read_ok message contains the broadcasted messages
    if let Some(reply_msg) = network.get_last_reply() {
        assert_eq!(reply_msg.body.get("type").unwrap(), "read_ok");
        assert_eq!(*reply_msg.body.get("messages").unwrap(), serde_json::json!([100, 200]));
    } else {
        panic!("Did not receive a reply for read message");
    }
}

#[tokio::test]
async fn test_topology_update() {
    let mut network = TestNetwork::new();
    network.add_node("node1".to_string(), "topology".to_string()).await;

    // Init node1
    let init_msg = make_init_msg();
    network.send_message(init_msg);
    while network.tick().await {}

    // Send topology message
    let topology_msg = make_topology_msg(1);
    network.send_message(topology_msg);
    while network.tick().await {}

    // Assert that node1's storage has the correct topology
    let node1_storage_arc = network.get_node_storage("node1");
    let node1_storage = node1_storage_arc.lock().await;
    assert!(node1_storage.topology.contains("node2"));
    assert!(node1_storage.topology.contains("node3"));
}

#[tokio::test]
async fn test_handle_broadcast() {
    let mut storage = Storage::new();
    let (tx, mut rx) = mpsc::channel(100);

    // Simulate init to set node_id
    storage.set_id("node1").await;

    // Handle a broadcast message
    handle_broadcast("client".to_string(), "node1".to_string(), 1, &mut storage, 123, tx.clone()).await.unwrap();

    // Assert storage is updated
    assert!(storage.values().contains(&123));

    // Assert broadcast_ok reply is sent
    let reply_str = rx.recv().await.unwrap();
    let reply: serde_json::Value = serde_json::from_str(&reply_str).unwrap();
    assert_eq!(reply["body"]["type"], "broadcast_ok");
    assert_eq!(reply["body"]["in_reply_to"], 1);
}

#[tokio::test]
async fn test_send_broadcast() {
    let (tx, mut rx) = mpsc::channel(100);

    send_broadcast("node1".to_string(), "node2".to_string(), 1, 123, tx.clone()).await.unwrap();

    let sent_msg_str = rx.recv().await.unwrap();
    let sent_msg: serde_json::Value = serde_json::from_str(&sent_msg_str).unwrap();

    assert_eq!(sent_msg["src"], "node1");
    assert_eq!(sent_msg["dest"], "node2");
    assert_eq!(sent_msg["body"]["type"], "broadcast");
    assert_eq!(sent_msg["body"]["msg_id"], 1);
    assert_eq!(sent_msg["body"]["message"], 123);
}

#[tokio::test]
async fn test_handle_broadcast_ok() {
    let mut storage = Storage::new();
    storage.set_id("node1").await;

    // Set up topology so node1 knows about node2
    storage.update_typology(vec!["node2".to_string()]);

    // Simulate a pending message
    storage.update_values("node1".to_string(), 456);
    let key = *storage.values.keys().next().unwrap();

    // Ensure it's pending for node2
    assert!(storage.peer_pending.get("node2").unwrap().contains(&key));

    // Handle broadcast_ok from node2
    handle_broadcast_ok("node2".to_string(), "node1".to_string(), key, &mut storage).await.unwrap();

    // Assert pending message is removed
    assert!(!storage.peer_pending.get("node2").unwrap().contains(&key));
}
