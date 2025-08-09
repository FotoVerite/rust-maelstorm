// tests/id_gen.rs
use std::collections::HashSet;
use std::thread;
use std::time::Duration;

use maelstrom_rust_node::state::State;
mod test_utils;
use maelstrom_rust_node::storage::Storage;
use test_utils::*;
#[test]
fn ids_are_unique_for_single_node() {
    let mut state = State::new();
    state.node_id = Some("node".to_string());
    let mut seen = HashSet::new();
    for _ in 0..10_000 {
        let id = state.next_id();
        assert!(seen.insert(id.clone()), "Duplicate ID generated: {}", id);
    }
}

#[test]
fn ids_increase_over_time() {
    let mut state = State::new();
    state.node_id = Some("node".to_string());
    let first = state.next_id();
    thread::sleep(Duration::from_millis(1));
    let second = state.next_id();

    assert!(second > first, "IDs did not increase over time");
}

#[test]
fn no_collision_between_two_nodes() {
    let mut state1 = State::new();
    state1.node_id = Some("node1".to_string());
    let mut state2 = State::new();
    state2.node_id = Some("node2".to_string());
    let mut seen = HashSet::new();

    for _ in 0..5_000 {
        let id1 = state1.next_id();
        let id2 = state2.next_id();
        assert!(seen.insert(id1), "Duplicate ID from node 1");
        assert!(seen.insert(id2), "Duplicate ID from node 2");
    }
}

#[test]
fn ordering_survives_same_millisecond() {
    let mut state = State::new();

    state.node_id = Some("node".to_string());

    let ids: Vec<String> = (0..1000).map(|_| state.next_id()).collect();

    let mut sorted = ids.clone();
    sorted.sort_unstable();

    assert_eq!(ids, sorted, "IDs out of order within same millisecond");
}

#[tokio::test]
async fn generate_id_after_init_returns_proper_response() {
    let mut state = State::new();

    // Send init to set node_id in state
    let init_msg = make_init_msg();
    let mut storage = Storage::new();

    let _ = run_test_message(&init_msg, &mut state, &mut storage).await;

    // Now send generate message with msg_id 10
    let gen_msg = make_generate_msg(10);
    let output = run_test_message(&gen_msg, &mut state, &mut storage).await;

    let resp: serde_json::Value = parse_reply(&output);
    let body = &resp["body"];

    assert_eq!(resp["src"], "node1");
    assert_eq!(resp["dest"], "client");
    assert_eq!(body["type"], "generate_ok");
    assert_eq!(body["in_reply_to"], 10);

    let id = body["id"].as_str().expect("id should be a string");
    assert!(
        id.starts_with("node1-"),
        "id must start with node_id prefix"
    );
}

#[tokio::test]
#[should_panic(expected = "Node id has not been sent")]
async fn generate_without_init_panics() {
    let mut state = State::new();

    let gen_msg = make_generate_msg(42);
    // This should panic because state.node_id is None
    let mut storage = Storage::new();

    let _ = run_test_message(&gen_msg, &mut state, &mut storage).await;
}

#[tokio::test]
async fn multiple_generate_ids_are_ordered_and_unique() {
    let mut state = State::new();

    // Initialize state with init message
    let init_msg = make_init_msg();
    let mut storage = Storage::new();

    let _ = run_test_message(&init_msg, &mut state, &mut storage).await;

    let mut ids = Vec::new();
    for i in 0..100 {
        let gen_msg = make_generate_msg(i);
        let output = run_test_message(&gen_msg, &mut state, &mut storage).await;
        let resp: serde_json::Value = parse_reply(&output);
        let id = resp["body"]["id"].as_str().unwrap().to_string();
        ids.push(id);
    }

    // Check ascending order by simple lex compare
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(ids, sorted, "IDs are not sorted");

    // Check uniqueness
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 100, "IDs are not unique");
}
