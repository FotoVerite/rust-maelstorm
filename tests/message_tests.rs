
use maelstrom_rust_node::state::State;
mod test_utils;

use test_utils::*;

#[test]
fn test_init_message() {
    let mut state = State::new();
    let init_msg = make_init_msg();

    let output = run_test_message(&init_msg, &mut state);
    assert!(output.contains("\"type\":\"init_ok\""));
}

#[test]
fn test_echo_message() {
    let mut state = State::new();
    state.current_workload = Some("echo".to_string());

    let echo_msg = make_echo_msg("hello");
    let output = run_test_message(&echo_msg, &mut state);

    assert!(output.contains("\"type\":\"echo_ok\""));
    assert!(output.contains("\"echo\":\"hello\""));
}
