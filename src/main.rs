use std::{
    collections::HashMap,
    
    sync::Arc,
};

use maelstrom_rust_node::{
    broadcast::actor::broadcast_message, process_message_line, storage::Storage, write_stdout,
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    sync::{Mutex, mpsc},
};
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (tx, rx) = mpsc::channel(1_000_000);
    let (gossip_sender, gossip_receiver) = mpsc::channel(1_000_000);

    let tx_read = tx.clone();
    let storage = Arc::new(Mutex::new(Storage::new(gossip_sender.clone())));
    let storage_read = Arc::clone(&storage);
    let read_stdin_task = {
        tokio::spawn(async move {
            let stdin = tokio::io::stdin();
            let reader = BufReader::new(stdin);
            let mut lines = reader.lines();
            let mut processed = HashMap::new();
            while let Some(line_res) = lines.next_line().await.transpose() {
                let line = line_res.expect("Failed to read line");
                _ = process_message_line(
                    line,
                    Arc::clone(&storage_read),
                    &mut processed,
                    tx_read.clone(),
                )
                .await;
            }
        })
    };

    let write_stdout_task = {
        tokio::spawn(async move {
            let stdout = tokio::io::stdout();

            _ = write_stdout(stdout, rx).await;
        })
    };

    let broadcast_read = Arc::clone(&storage);

    let broadcast_message_sender = tokio::spawn(async move {
        let _ = broadcast_message(gossip_receiver, broadcast_read, tx, ).await;
    });

    let (reader_result, writer_result, _) =
        tokio::join!(read_stdin_task, write_stdout_task, broadcast_message_sender);

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
