use maelstrom_rust_node::{
    state::State,
    storage::{Storage},
};
mod test_utils;

use test_utils::*;

#[test]
fn test_broadcast_and_read() {
    let mut state = State::new();
    let mut storage = Storage::new();

    state.current_workload = Some("broadcast".to_string());
    state.node_id = Some("node1".to_string());

    let broadcast_msg = make_broadcast_msg(1, 123);
    let output = run_test_message(&broadcast_msg, &mut state, &mut storage);
    assert_response(&output, "broadcast_ok", |_| {});

    let read_msg = make_read_msg(2);
    let output = run_test_message(&read_msg, &mut state, &mut storage);
    assert_response(&output, "read_ok", |body| {
        assert_eq!(body["messages"], serde_json::json!([123]));
    });
}

#[test]
fn test_read_empty() {
    let mut state = State::new();
    let mut storage = Storage::new();

    state.current_workload = Some("broadcast".to_string());
    state.node_id = Some("node1".to_string());

    let read_msg = make_read_msg(1);
    let output = run_test_message(&read_msg, &mut state, &mut storage);
    assert_response(&output, "read_ok", |body| {
        assert_eq!(body["messages"], serde_json::json!([]));
    });
}

#[test]
fn test_multiple_broadcasts() {
    let mut state = State::new();
    let mut storage = Storage::new();

    state.current_workload = Some("broadcast".to_string());
    state.node_id = Some("node1".to_string());

    let broadcast_msg1 = make_broadcast_msg(1, 123);
    let _ = run_test_message(&broadcast_msg1, &mut state, &mut storage);

    let broadcast_msg2 = make_broadcast_msg(2, 456);
    let _ = run_test_message(&broadcast_msg2, &mut state, &mut storage);

    let read_msg = make_read_msg(3);
    let output = run_test_message(&read_msg, &mut state, &mut storage);
    assert_response(&output, "read_ok", |body| {
        assert_eq!(body["messages"], serde_json::json!([123, 456]));
    });
}
