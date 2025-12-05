//! OpenAI function calling schema conversion.

use crate::schema::{ToolSchema, ToolSchemaConverter};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// OpenAI function schema format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIToolSchema {
    /// Type (always "function")
    #[serde(rename = "type")]
    pub tool_type: String,
    /// Function details
    pub function: OpenAIFunction,
}

/// OpenAI function definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunction {
    /// Function name
    pub name: String,
    /// Function description
    pub description: String,
    /// Parameters schema
    pub parameters: Value,
}

impl ToolSchemaConverter for OpenAIToolSchema {
    type Output = Self;

    fn convert(schema: &ToolSchema) -> Self::Output {
        Self {
            tool_type: "function".to_string(),
            function: OpenAIFunction {
                name: schema.name.clone(),
                description: schema.description.clone(),
                parameters: schema.input_schema.clone(),
            },
        }
    }
}
