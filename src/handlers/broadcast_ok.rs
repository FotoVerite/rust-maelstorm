use std::sync::Arc;

use tokio::sync::Mutex;

use crate::storage::Storage;

pub async fn handle_broadcast_ok(
    _src: String,
    dest: String,
    in_reply_to: u64,
    storage: Arc<Mutex<Storage>>,
) -> anyhow::Result<()> {
    tokio::spawn(async move {
        let storage = storage.lock().await;
        storage.values.remove_pending(dest, in_reply_to).await;
    });
    Ok(())
}