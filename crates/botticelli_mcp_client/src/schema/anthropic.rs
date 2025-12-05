//! Anthropic tool schema conversion.

use crate::schema::{ToolSchema, ToolSchemaConverter};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Anthropic-specific tool schema format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicToolSchema {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Input schema
    pub input_schema: Value,
}

impl ToolSchemaConverter for AnthropicToolSchema {
    type Output = Self;

    fn convert(schema: &ToolSchema) -> Self::Output {
        Self {
            name: schema.name.clone(),
            description: schema.description.clone(),
            input_schema: schema.input_schema.clone(),
        }
    }
}
