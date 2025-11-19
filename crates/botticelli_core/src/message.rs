//! Message types for conversation history.

use crate::{Input, Role};
use serde::{Deserialize, Serialize};

/// A multimodal message in a conversation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: Vec<Input>,
}
