//! Media source types for multimodal content.

use serde::{Deserialize, Serialize};

/// Where media content is sourced from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MediaSource {
    /// URL to fetch the content from
    Url(String),
    /// Base64-encoded content
    Base64(String),
    /// Raw binary data
    Binary(Vec<u8>),
}
