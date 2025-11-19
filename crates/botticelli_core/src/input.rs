//! Input types for LLM requests.

use crate::MediaSource;
use serde::{Deserialize, Serialize};

/// Supported input types to LLMs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Input {
    /// Plain text input.
    Text(String),

    /// Image input (PNG, JPEG, WebP, GIF, etc.).
    Image {
        /// MIME type, e.g., "image/png" or "image/jpeg"
        mime: Option<String>,
        /// Media source (URL, base64, or raw bytes)
        source: MediaSource,
    },

    /// Audio input (MP3, WAV, OGG, etc.).
    Audio {
        /// MIME type, e.g., "audio/mp3" or "audio/wav"
        mime: Option<String>,
        /// Media source (URL, base64, or raw bytes)
        source: MediaSource,
    },

    /// Video input (MP4, WebM, AVI, etc.).
    Video {
        /// MIME type, e.g., "video/mp4" or "video/webm"
        mime: Option<String>,
        /// Media source (URL, base64, or raw bytes)
        source: MediaSource,
    },

    /// Document input (PDF, DOCX, TXT, etc.).
    Document {
        /// MIME type, e.g., "application/pdf" or "text/plain"
        mime: Option<String>,
        /// Media source (URL, base64, or raw bytes)
        source: MediaSource,
        /// Optional filename for context
        filename: Option<String>,
    },
}
