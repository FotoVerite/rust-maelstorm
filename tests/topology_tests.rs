use maelstrom_rust_node::{
    state::State,
    storage::{Storage},
};
mod test_utils;

use test_utils::*;

#[tokio::test]
async fn test_topology() {
    let mut state = State::new();
    let mut storage = Storage::new();

    state.node_id = Some("node1".to_string());

    let topology_msg = make_topology_msg(1);
    let output = run_test_message(&topology_msg, &mut state, &mut storage).await;
    assert_response(&output, "topology_ok", |_| {});

    assert!(storage.topology.contains("node2"));
    assert!(storage.topology.contains("node3"));
}
