
use crate::{storage::Storage};

pub async fn handle_error(
    in_reply_to: u64,
    storage: &mut Storage,
) -> anyhow::Result<()> {
    storage.retry_for_cas(in_reply_to).await;
   
    Ok(())
}
