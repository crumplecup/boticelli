//! Groq function calling schema conversion (OpenAI-compatible).

use crate::schema::{ToolSchema, ToolSchemaConverter};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Groq tool schema format (OpenAI-compatible).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroqToolSchema {
    /// Type (always "function")
    #[serde(rename = "type")]
    pub tool_type: String,
    /// Function details
    pub function: GroqFunction,
}

/// Groq function definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroqFunction {
    /// Function name
    pub name: String,
    /// Function description
    pub description: String,
    /// Parameters schema
    pub parameters: Value,
}

impl ToolSchemaConverter for GroqToolSchema {
    type Output = Self;

    fn convert(schema: &ToolSchema) -> Self::Output {
        Self {
            tool_type: "function".to_string(),
            function: GroqFunction {
                name: schema.name.clone(),
                description: schema.description.clone(),
                parameters: schema.input_schema.clone(),
            },
        }
    }
}
