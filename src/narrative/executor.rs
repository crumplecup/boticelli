//! Narrative execution logic.
//!
//! This module provides the executor that processes multi-act narratives
//! by calling LLM APIs in sequence, passing context between acts.

use crate::{BoticelliDriver, GenerateRequest, Input, Message, Output, Role};
use crate::{BoticelliResult, Narrative};
use serde::{Deserialize, Serialize};

/// Execution result for a single act in a narrative.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ActExecution {
    /// Name of the act (from the narrative).
    pub act_name: String,

    /// The prompt that was sent to the LLM.
    pub prompt: String,

    /// The text response from the LLM.
    pub response: String,

    /// Position in the execution sequence (0-indexed).
    pub sequence_number: usize,
}

/// Complete execution result for a narrative.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NarrativeExecution {
    /// Name of the narrative that was executed.
    pub narrative_name: String,

    /// Ordered list of act executions.
    pub act_executions: Vec<ActExecution>,
}

/// Executes narratives by calling LLM APIs in sequence.
///
/// The executor processes each act in the narrative's table of contents order,
/// passing previous act outputs as context to subsequent acts.
pub struct NarrativeExecutor<D: BoticelliDriver> {
    driver: D,
}

impl<D: BoticelliDriver> NarrativeExecutor<D> {
    /// Create a new narrative executor with the given LLM driver.
    pub fn new(driver: D) -> Self {
        Self { driver }
    }

    /// Execute a narrative, processing all acts in sequence.
    ///
    /// Each act sees the outputs from all previous acts as conversation history.
    /// The first act receives just its prompt, the second act sees the first act's
    /// response plus its own prompt, and so on.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Any LLM API call fails
    /// - The response format is unexpected
    pub async fn execute(&self, narrative: &Narrative) -> BoticelliResult<NarrativeExecution> {
        let mut act_executions = Vec::new();
        let mut conversation_history: Vec<Message> = Vec::new();

        for (sequence_number, act_name) in narrative.toc.order.iter().enumerate() {
            // Get the prompt for this act
            let prompt = narrative.acts.get(act_name)
                .expect("Narrative validation should ensure all acts exist");

            // Build the request with conversation history + current prompt
            conversation_history.push(Message {
                role: Role::User,
                content: vec![Input::Text(prompt.clone())],
            });

            let request = GenerateRequest {
                messages: conversation_history.clone(),
                max_tokens: None,
                temperature: None,
                model: None,
            };

            // Call the LLM
            let response = self.driver.generate(&request).await?;

            // Extract text from response
            let response_text = extract_text_from_outputs(&response.outputs)?;

            // Store the act execution
            act_executions.push(ActExecution {
                act_name: act_name.clone(),
                prompt: prompt.clone(),
                response: response_text.clone(),
                sequence_number,
            });

            // Add the assistant's response to conversation history for the next act
            conversation_history.push(Message {
                role: Role::Assistant,
                content: vec![Input::Text(response_text)],
            });
        }

        Ok(NarrativeExecution {
            narrative_name: narrative.metadata.name.clone(),
            act_executions,
        })
    }

    /// Get a reference to the underlying LLM driver.
    pub fn driver(&self) -> &D {
        &self.driver
    }
}

/// Extract text content from LLM outputs.
///
/// Concatenates all text outputs with newlines between them.
fn extract_text_from_outputs(outputs: &[Output]) -> BoticelliResult<String> {
    let mut texts = Vec::new();

    for output in outputs {
        if let Output::Text(text) = output {
            texts.push(text.clone());
        }
    }

    if texts.is_empty() {
        Ok(String::new())
    } else {
        Ok(texts.join("\n"))
    }
}
