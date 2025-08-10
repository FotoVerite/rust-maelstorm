
use crate::{
    storage::Storage,
};

pub async fn handle_broadcast_ok (
    src: String,
        _dest: String,
    in_reply_to: u64,
    storage: &mut Storage,
) -> anyhow::Result<()> {
    storage.remove_from_peer_pending(src, in_reply_to);
    Ok(())
}
