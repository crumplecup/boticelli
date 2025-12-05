//! Gemini function calling schema conversion.

use crate::schema::{ToolSchema, ToolSchemaConverter};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Gemini function declaration format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiToolSchema {
    /// Function name
    pub name: String,
    /// Function description
    pub description: String,
    /// Parameters schema
    pub parameters: Value,
}

impl ToolSchemaConverter for GeminiToolSchema {
    type Output = Self;

    fn convert(schema: &ToolSchema) -> Self::Output {
        Self {
            name: schema.name.clone(),
            description: schema.description.clone(),
            parameters: schema.input_schema.clone(),
        }
    }
}
