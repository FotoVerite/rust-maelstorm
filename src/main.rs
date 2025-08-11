use std::{
    io::{self},
    sync::Arc,
};

use maelstrom_rust_node::{
    broadcast::actor::broadcast_message,
    process_message_line,
    storage::{value_store::spawn_gossip_sender, Storage},
    write_stdout,
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    sync::{Mutex, mpsc},
};
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (tx, rx) = mpsc::channel(1024);
    let (gossip_sender, gossip_receiver) = mpsc::channel(1024);

    let tx_read = tx.clone();
    let storage = Arc::new(Mutex::new(Storage::new(gossip_sender.clone())));
    let storage_read = Arc::clone(&storage);
    let node_id_arc = Arc::clone(&storage_read.lock().await.node_id);
    let read_stdin_task = {
        tokio::spawn(async move {
            let stdin = tokio::io::stdin();
            let reader = BufReader::new(stdin);
            let mut lines = reader.lines();

            while let Some(line_res) = lines.next_line().await.transpose() {
                let line = line_res.expect("Failed to read line");
                let mut storage_guard = storage_read.lock().await;
                _ = process_message_line(line, &mut *storage_guard, tx_read.clone()).await;
            }
        })
    };

    let write_stdout_task = {
        tokio::spawn(async move {
            let stdout = io::stdout();

            _ = write_stdout(stdout, rx).await;
        })
    };

    let gossip_sender = tokio::spawn(async move {
        spawn_gossip_sender(Arc::clone(&storage), gossip_sender).await;
    });
    let broadcast_message_sender = tokio::spawn(async move {
        let _ = broadcast_message(gossip_receiver, tx, node_id_arc).await;
    });

    let (reader_result, writer_result, _, _) = tokio::join!(
        read_stdin_task,
        write_stdout_task,
        gossip_sender,
        broadcast_message_sender
    );

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
