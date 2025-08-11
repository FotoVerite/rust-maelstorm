use tokio::sync::mpsc::Sender;

use crate::{storage::Storage};

pub async fn handle_cas_ok(
    _src: String,
    in_reply_to: u64,
    storage: &mut Storage,
    _tx: Sender<String>,
) -> anyhow::Result<()> {
    storage.remove_request_from_pending_cas(in_reply_to);
    Ok(())
}
