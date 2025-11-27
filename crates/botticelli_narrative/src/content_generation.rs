//! Content generation processor for creating reviewable content.
//!
//! This processor detects narratives with a `template` field and generates
//! content into custom tables based on Discord schema templates, OR infers
//! schema automatically from JSON responses when no template is provided.

use crate::{
    ActProcessor, CompleteGeneration, CreateTableFromInference, CreateTableFromTemplate,
    InsertContent, ProcessorContext, StartGeneration, extraction::extract_json,
    extraction::parse_json,
};
use actix::Addr;
use async_trait::async_trait;
use botticelli_error::BotticelliResult;
use serde_json::Value as JsonValue;

/// Content generation processing mode
#[derive(Debug, Clone, PartialEq)]
enum ProcessingMode {
    /// Use an explicit template to define table schema
    Template(String),
    /// Infer schema automatically from JSON response
    Inference,
}

/// Processor for content generation into custom tables.
///
/// Detects narratives with a `template` field and:
/// 1. Creates a custom table if it doesn't exist (based on template schema)
/// 2. Tracks generation start in content_generations table
/// 3. Extracts JSON from LLM responses
/// 4. Inserts generated content with metadata columns
/// 5. Updates tracking record with success/failure
pub struct ContentGenerationProcessor {
    /// Storage actor address for asynchronous database operations
    storage_actor: Addr<crate::StorageActor>,
}

impl ContentGenerationProcessor {
    /// Create a new content generation processor with storage actor.
    ///
    /// # Arguments
    ///
    /// * `storage_actor` - Address of the storage actor for database operations
    pub fn new(storage_actor: Addr<crate::StorageActor>) -> Self {
        Self { storage_actor }
    }
}

#[async_trait]
impl ActProcessor for ContentGenerationProcessor {
    async fn process(&self, context: &ProcessorContext<'_>) -> BotticelliResult<()> {

        // Determine processing mode: Template or Inference
        let processing_mode = if let Some(template) = &context.narrative_metadata.template() {
            ProcessingMode::Template(template.clone())
        } else {
            ProcessingMode::Inference
        };

        // Use target if specified, otherwise template name or narrative name
        let table_name = if let Some(target) = context.narrative_metadata.target() {
            target.to_string()
        } else {
            match &processing_mode {
                ProcessingMode::Template(template) => template.clone(),
                ProcessingMode::Inference => context.narrative_metadata.name().to_string(),
            }
        };

        tracing::info!(
            act = %context.execution.act_name,
            table = %table_name,
            mode = ?processing_mode,
            "Processing content generation"
        );

        let start_time = std::time::Instant::now();

        // Track generation start (fire and forget - don't block on tracking failures)
        let _ = self
            .storage_actor
            .send(StartGeneration {
                table_name: table_name.clone(),
                narrative_file: format!("{} (from processor)", context.narrative_name),
                narrative_name: context.narrative_name.to_string(),
            })
            .await;

        // Execute content generation
        let generation_result: Result<usize, botticelli_error::BotticelliError> = async {
            // Extract JSON from response first (needed for both modes)
            let json_str = extract_json(&context.execution.response)?;

            tracing::debug!(json_length = json_str.len(), "Extracted JSON from response");

            // Parse JSON - could be single object or array
            let parsed_json: JsonValue = parse_json(&json_str)?;

            let items: Vec<JsonValue> = if parsed_json.is_array() {
                parsed_json.as_array().unwrap().to_vec()
            } else {
                vec![parsed_json.clone()]
            };

            // Create table based on processing mode
            match &processing_mode {
                ProcessingMode::Template(template) => {
                    self.storage_actor
                        .send(CreateTableFromTemplate {
                            table_name: table_name.clone(),
                            template: template.clone(),
                            narrative_name: Some(context.narrative_name.to_string()),
                            description: Some(context.narrative_metadata.description().to_string()),
                        })
                        .await
                        .map_err(|e| {
                            botticelli_error::BackendError::new(format!(
                                "Failed to send create table message: {}",
                                e
                            ))
                        })?
                        .map_err(|e| {
                            botticelli_error::BackendError::new(format!(
                                "Failed to create table from template: {}",
                                e
                            ))
                        })?;
                }
                ProcessingMode::Inference => {
                    self.storage_actor
                        .send(CreateTableFromInference {
                            table_name: table_name.clone(),
                            json_sample: parsed_json.clone(),
                            narrative_name: Some(context.narrative_name.to_string()),
                            description: Some(context.narrative_metadata.description().to_string()),
                        })
                        .await
                        .map_err(|e| {
                            botticelli_error::BackendError::new(format!(
                                "Failed to send create table message: {}",
                                e
                            ))
                        })?
                        .map_err(|e| {
                            botticelli_error::BackendError::new(format!(
                                "Failed to create table from inference: {}",
                                e
                            ))
                        })?;
                }
            }

            tracing::info!(
                count = items.len(),
                table = %table_name,
                "Parsed JSON items for insertion"
            );

            // Insert each item
            for (idx, item) in items.iter().enumerate() {
                tracing::debug!(
                    index = idx,
                    act = %context.execution.act_name,
                    "Inserting content item"
                );

                self.storage_actor
                    .send(InsertContent {
                        table_name: table_name.clone(),
                        json_data: item.clone(),
                        narrative_name: context.narrative_name.to_string(),
                        act_name: context.execution.act_name.clone(),
                        model: context.execution.model.clone(),
                    })
                    .await
                    .map_err(|e| {
                        botticelli_error::BackendError::new(format!(
                            "Failed to send insert message: {}",
                            e
                        ))
                    })?
                    .map_err(|e| {
                        botticelli_error::BackendError::new(format!("Failed to insert content: {}", e))
                    })?;
            }

            Ok(items.len())
        }
        .await;

        // Update tracking record with result
        let duration_ms = start_time.elapsed().as_millis() as i32;

        let complete_msg = match &generation_result {
            Ok(row_count) => CompleteGeneration {
                table_name: table_name.clone(),
                row_count: Some(*row_count as i32),
                duration_ms,
                status: "success".to_string(),
                error_message: None,
            },
            Err(e) => CompleteGeneration {
                table_name: table_name.clone(),
                row_count: None,
                duration_ms,
                status: "failed".to_string(),
                error_message: Some(e.to_string()),
            },
        };

        // Fire and forget - don't block on tracking update
        let _ = self.storage_actor.send(complete_msg).await;

        // Return the original result
        generation_result.map(|row_count| {
            tracing::info!(
                act = %context.execution.act_name,
                table = %table_name,
                count = row_count,
                "Content generation completed successfully"
            );
        })
    }

    fn should_process(&self, context: &ProcessorContext<'_>) -> bool {
        // Don't process if user explicitly opted out
        if *context.narrative_metadata.skip_content_generation() {
            return false;
        }

        // Otherwise, process (with template OR inference mode)
        true
    }

    fn name(&self) -> &str {
        "ContentGenerationProcessor"
    }
}
