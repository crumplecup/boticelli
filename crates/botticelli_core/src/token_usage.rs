//! Token usage tracking for LLM requests.

use serde::{Deserialize, Serialize};

/// Token usage information for a completed generation.
///
/// This provides a unified view of token consumption across different
/// LLM providers, which may report tokens differently.
///
/// # Examples
///
/// ```
/// use botticelli_core::TokenUsageData;
///
/// let usage = TokenUsageData::new(150, 50, 200);
/// assert_eq!(usage.input_tokens(), 150);
/// assert_eq!(usage.output_tokens(), 50);
/// assert_eq!(usage.total_tokens(), 200);
/// ```
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    derive_getters::Getters,
    derive_builder::Builder,
)]
pub struct TokenUsageData {
    /// Number of tokens in the input/prompt.
    input_tokens: u64,
    /// Number of tokens in the generated output.
    output_tokens: u64,
    /// Total tokens consumed (may differ from input + output due to provider accounting).
    total_tokens: u64,
}

impl TokenUsageData {
    /// Creates new token usage data.
    pub fn new(input_tokens: u64, output_tokens: u64, total_tokens: u64) -> Self {
        Self {
            input_tokens,
            output_tokens,
            total_tokens,
        }
    }

    /// Creates a builder for TokenUsageData.
    pub fn builder() -> TokenUsageDataBuilder {
        TokenUsageDataBuilder::default()
    }
}
