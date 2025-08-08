use std::io::{self, BufRead};

use maelstrom_rust_node::{
    process_message_line, state::State
};

fn main() {
    let mut state = State::new();
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        let line = line.expect("Failed to read line");
        process_message_line(line, &mut out);
    }
}
