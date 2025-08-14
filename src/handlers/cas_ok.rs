use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{storage::Storage};

pub async fn handle_cas_ok(
    _src: String,
    in_reply_to: u64,
    storage: Arc<Mutex<Storage>>,
) -> anyhow::Result<()> {
    tokio::spawn(async move {
        let mut storage = storage.lock().await;
        let _ = storage.g_counter.remove_from_pending(in_reply_to);
    });
    Ok(())
}
