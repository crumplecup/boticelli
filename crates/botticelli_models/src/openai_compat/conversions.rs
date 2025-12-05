//! Type conversions between Botticelli and OpenAI formats.

use crate::openai_compat::{ChatMessage, ChatRequest, ChatResponse, OpenAICompatError};
use botticelli_core::{GenerateRequest, GenerateResponse, Input, Output};

/// Converts a Botticelli GenerateRequest to OpenAI chat format.
pub fn to_chat_request(
    req: &GenerateRequest,
    model: &str,
) -> Result<ChatRequest, OpenAICompatError> {
    let mut messages = Vec::new();

    for msg in req.messages() {
        let role = match msg.role() {
            botticelli_core::Role::User => "user",
            botticelli_core::Role::Assistant => "assistant",
            botticelli_core::Role::System => "system",
        };

        for content in msg.content() {
            match content {
                Input::Text(text) => {
                    messages.push(ChatMessage {
                        role: role.to_string(),
                        content: text.clone(),
                    });
                }
                _ => {
                    return Err(OpenAICompatError::InvalidRequest(
                        "Only text inputs supported in OpenAI format".to_string(),
                    ));
                }
            }
        }
    }

    let mut builder = ChatRequest::builder();
    builder.model(model.to_string()).messages(messages);

    if let Some(max_tokens) = req.max_tokens() {
        builder.max_tokens(*max_tokens);
    }

    if let Some(temp) = req.temperature() {
        builder.temperature(*temp);
    }

    builder
        .build()
        .map_err(|e| OpenAICompatError::Builder(format!("Failed to build request: {}", e)))
}

/// Converts an OpenAI chat response to Botticelli GenerateResponse.
pub fn from_chat_response(response: &ChatResponse) -> Result<GenerateResponse, OpenAICompatError> {
    let content = response
        .choices
        .first()
        .map(|choice| choice.message.content.clone())
        .ok_or_else(|| OpenAICompatError::ResponseParsing("No choices in response".to_string()))?;

    let output = Output::Text(content);

    GenerateResponse::builder()
        .outputs(vec![output])
        .build()
        .map_err(|e| OpenAICompatError::Builder(format!("Failed to build response: {}", e)))
}
