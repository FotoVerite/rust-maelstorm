use crate::state::State;
use crate::message::Message;

pub mod state;
pub mod message;
pub mod handlers;

pub trait Handler {
    /// Handle an incoming message, possibly mutating state, and produce zero or more responses.
    fn handle(&mut self, msg: &Message, state: &mut State) -> Vec<Message>;
}