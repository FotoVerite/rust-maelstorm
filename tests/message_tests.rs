use maelstrom_rust_node::{
    state::State,
    storage::{Storage},
};
mod test_utils;

use test_utils::*;

#[tokio::test]
async fn test_init_message() {
    let mut state = State::new();
    let mut storage = Storage::new();
    let init_msg = make_init_msg();

    let output = run_test_message(&init_msg, &mut state, &mut storage).await;
    assert_response(&output, "init_ok", |_| {});
}

#[tokio::test]
async fn test_echo_message() {
    let mut state = State::new();
    let mut storage = Storage::new();

    state.current_workload = Some("echo".to_string());

    let echo_msg = make_echo_msg("hello");
    let output = run_test_message(&echo_msg, &mut state, &mut storage).await;
    assert_response(&output, "echo_ok", |body| {
        assert_eq!(body["echo"], "hello");
    });
}