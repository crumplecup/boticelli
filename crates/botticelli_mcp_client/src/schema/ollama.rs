//! Ollama tool schema conversion (prompt engineering approach).

use crate::schema::{ToolSchema, ToolSchemaConverter};

/// Ollama tool representation (prompt-based).
#[derive(Debug, Clone)]
pub struct OllamaToolSchema {
    /// Tool prompt representation
    pub prompt: String,
}

impl ToolSchemaConverter for OllamaToolSchema {
    type Output = Self;

    fn convert(schema: &ToolSchema) -> Self::Output {
        // Convert tool schema to prompt format
        let prompt = format!(
            "Tool: {}\nDescription: {}\nParameters: {}",
            schema.name,
            schema.description,
            serde_json::to_string_pretty(&schema.input_schema).unwrap_or_default()
        );

        Self { prompt }
    }
}

impl OllamaToolSchema {
    /// Generate system prompt with available tools.
    pub fn generate_system_prompt(tools: &[ToolSchema]) -> String {
        let tool_descriptions: Vec<String> = tools
            .iter()
            .map(|tool| {
                format!(
                    "- {} ({}): {}",
                    tool.name,
                    serde_json::to_string(&tool.input_schema).unwrap_or_default(),
                    tool.description
                )
            })
            .collect();

        format!(
            "You have access to the following tools:\n\n{}\n\n\
            To use a tool, respond with JSON in this format:\n\
            {{\"tool\": \"tool_name\", \"arguments\": {{\"param\": \"value\"}}}}\n\n\
            Only use tools when necessary. Provide direct responses when possible.",
            tool_descriptions.join("\n")
        )
    }
}
