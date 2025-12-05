//! HuggingFace tool schema conversion (model-dependent).

use crate::schema::{ToolSchema, ToolSchemaConverter};
use serde_json::Value;

/// HuggingFace tool representation.
#[derive(Debug, Clone)]
pub struct HuggingFaceToolSchema {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Parameters
    pub parameters: Value,
    /// Formatted prompt representation
    pub prompt_format: String,
}

impl ToolSchemaConverter for HuggingFaceToolSchema {
    type Output = Self;

    fn convert(schema: &ToolSchema) -> Self::Output {
        // Format tool as prompt (model-dependent, may need templates)
        let prompt_format = format!(
            "Function: {}\nDescription: {}\nInput: {}",
            schema.name,
            schema.description,
            serde_json::to_string_pretty(&schema.input_schema).unwrap_or_default()
        );

        Self {
            name: schema.name.clone(),
            description: schema.description.clone(),
            parameters: schema.input_schema.clone(),
            prompt_format,
        }
    }
}

impl HuggingFaceToolSchema {
    /// Generate tool calling prompt for HuggingFace models.
    pub fn generate_tool_prompt(tools: &[ToolSchema]) -> String {
        let tool_list: Vec<String> = tools
            .iter()
            .map(|tool| format!("- {}: {}", tool.name, tool.description))
            .collect();

        format!(
            "Available functions:\n{}\n\n\
            Respond with JSON to call a function: {{\"function\": \"name\", \"args\": {{...}}}}\n\
            Or respond normally if no function call is needed.",
            tool_list.join("\n")
        )
    }
}
