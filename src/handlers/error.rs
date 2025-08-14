
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::{storage::Storage};

pub async fn handle_error(
    in_reply_to: u64,
    storage: Arc<Mutex<Storage>>,
) -> anyhow::Result<()> {
    tokio::spawn(async move {
        let mut storage = storage.lock().await;
        let _ = storage.g_counter.retry_for_cas(in_reply_to);
    });
    Ok(())
}
