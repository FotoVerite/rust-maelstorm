use std::{
    io::{self, BufRead},
    sync::Arc,
};

use maelstrom_rust_node::{process_message_line, state::State, storage::Storage, write_stdout};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    sync::{Mutex, mpsc},
};
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = Arc::new(Mutex::new(State::new()));
    let storage = Arc::new(Mutex::new(Storage::new()));
    let (tx, rx) = mpsc::channel(1024);
    let state_read = Arc::clone(&state);
    let storage_read = Arc::clone(&storage);
    let tx_read = tx.clone();
    let read_stdin_task = {
        tokio::spawn(async move {
            let stdin = tokio::io::stdin();
            let reader = BufReader::new(stdin);
            let mut lines = reader.lines();

            while let Some(line_res) = lines.next_line().await.transpose() {
                let line = line_res.expect("Failed to read line");
                let mut state_guard = state_read.lock().await;
                let mut storage_guard = storage_read.lock().await;
                _ = process_message_line(
                    line,
                    &mut *state_guard,
                    &mut *storage_guard,
                    tx_read.clone(),
                )
                .await;
            }
        })
    };

    let write_stdout_task = {
        tokio::spawn(async move {
            let stdout = io::stdout();

            _ = write_stdout(stdout, rx).await;
        })
    };

    let (reader_result, writer_result) = tokio::join!(read_stdin_task, write_stdout_task);

    // Handle results properly
    match reader_result {
        Result::Ok(()) => {}

        Err(e) => eprintln!("Reader task panicked: {:?}", e),
    }

    match writer_result {
        Result::Ok(()) => {}
        Err(e) => eprintln!("Writer task panicked: {:?}", e),
    }
    Ok(())
}
