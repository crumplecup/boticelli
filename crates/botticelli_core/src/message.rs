//! Message types for conversation history.

use crate::{Input, Role};
use serde::{Deserialize, Serialize};

/// A multimodal message in a conversation.
///
/// # Examples
///
/// ```
/// use botticelli_core::{Message, Role, Input};
///
/// let message = Message::builder()
///     .role(Role::User)
///     .content(vec![Input::Text("Hello!".to_string())])
///     .build()
///     .unwrap();
///
/// assert_eq!(*message.role(), Role::User);
/// assert_eq!(message.content().len(), 1);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_getters::Getters, derive_builder::Builder)]
pub struct Message {
    /// The role of the message sender
    role: Role,
    /// The content of the message (can be multimodal)
    content: Vec<Input>,
}
