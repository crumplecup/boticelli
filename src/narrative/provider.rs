//! Trait abstraction for narrative configuration providers.
//!
//! This module defines the `NarrativeProvider` trait, which decouples the
//! narrative executor from specific configuration formats (TOML, YAML, JSON, etc.).

/// Provides access to narrative configuration data.
///
/// This trait abstracts over different configuration sources (TOML files,
/// YAML, JSON, databases, etc.), allowing the executor to work with any
/// implementation.
///
/// By programming to this interface rather than concrete types, we achieve:
/// - Format flexibility (easy to add new config formats)
/// - Better testability (simple mock implementations)
/// - Reduced coupling (config changes don't ripple through executor)
pub trait NarrativeProvider {
    /// Name of the narrative for tracking and identification.
    fn name(&self) -> &str;

    /// Ordered list of act names to execute in sequence.
    ///
    /// The executor will process acts in this exact order.
    fn act_names(&self) -> &[String];

    /// Get the prompt text for a specific act.
    ///
    /// Returns `None` if the act doesn't exist.
    fn get_act_prompt(&self, act_name: &str) -> Option<&str>;
}
