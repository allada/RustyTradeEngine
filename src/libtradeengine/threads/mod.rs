use engine;
use std::vec::Vec;

extern crate uuid;

pub mod io;
pub mod matcher;

pub type ActionIdT = uuid::Bytes;

/// Action message to transport to matcher thread for processing.
/// This will always cause the matcher thread to respond with an
/// IoThreadMessage::MatcherActionResult with the same `action_id`
/// which will contain the results of the action.
#[derive(Debug)]
pub struct MatcherThreadActionMessage {
    pub action_id: ActionIdT,
    pub action: engine::MatcherAction,
}

/// Result from a MatcherThreadActionMessage. The `action_id`
/// will correspond to an `action_id` sent from a previous
/// MatcherThreadActionMessage.
#[derive(Debug)]
pub struct MatcherActionResponse {
    pub action_id: ActionIdT,
    pub result: engine::MatcherActionResult,
}

// Result<Vec<LedgerMutation>, LedgerActionError>

/// All messages received on the IO thread must be defined here.
pub enum IoThreadMessage {
    ProcessRawData(Vec<u8>),
    MatcherActionResult(MatcherActionResponse),
    Shutdown,
}
