//! Core data types for the Botticelli LLM API library.
//!
//! This crate provides the foundation data types used across all Botticelli interfaces.

mod role;
mod media;
mod input;
mod output;
mod message;
mod request;

pub use role::Role;
pub use media::MediaSource;
pub use input::Input;
pub use output::{Output, ToolCall};
pub use message::Message;
pub use request::{GenerateRequest, GenerateResponse};
