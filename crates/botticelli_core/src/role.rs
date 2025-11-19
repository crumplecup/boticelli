//! Role types for conversation participants.

use serde::{Deserialize, Serialize};

/// Roles are the same across modalities (text, image, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Role {
    System,
    User,
    Assistant,
}
