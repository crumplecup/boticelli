use crate::config::GenerationConfig;
use crate::metrics::BotMetrics;
use botticelli_interface::BotticelliDriver;
use botticelli_narrative::NarrativeExecutor;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use tracing::{debug, error, info, instrument};

/// Message types for generation bot.
#[derive(Debug)]
pub enum GenerationMessage {
    /// Trigger content generation
    Generate,
    /// Shutdown the bot
    Shutdown,
}

/// Bot that generates content on a schedule.
pub struct GenerationBot<D: BotticelliDriver> {
    config: GenerationConfig,
    executor: Arc<NarrativeExecutor<D>>,
    metrics: Arc<BotMetrics>,
    rx: mpsc::Receiver<GenerationMessage>,
}

impl<D: BotticelliDriver> GenerationBot<D> {
    /// Creates a new generation bot.
    pub fn new(
        config: GenerationConfig,
        executor: Arc<NarrativeExecutor<D>>,
        metrics: Arc<BotMetrics>,
        rx: mpsc::Receiver<GenerationMessage>,
    ) -> Self {
        Self {
            config,
            executor,
            metrics,
            rx,
        }
    }

    /// Runs the generation bot loop.
    #[instrument(skip(self))]
    pub async fn run(mut self) {
        info!("Generation bot started");

        while let Some(msg) = self.rx.recv().await {
            match msg {
                GenerationMessage::Generate => {
                    if let Err(e) = self.generate_content().await {
                        error!(error = ?e, "Content generation failed");
                    }
                }
                GenerationMessage::Shutdown => {
                    info!("Generation bot shutting down");
                    break;
                }
            }
        }
    }

    #[instrument(skip(self))]
    async fn generate_content(&self) -> Result<(), Box<dyn std::error::Error>> {
        let start = Instant::now();
        self.metrics.record_generation_execution();

        debug!(
            narrative = %self.config.narrative_name,
            "Starting content generation"
        );

        let result = self
            .executor
            .execute_narrative_by_name(
                &self.config.narrative_path.to_string_lossy(),
                &self.config.narrative_name,
            )
            .await;

        let duration = start.elapsed();

        match result {
            Ok(_) => {
                self.metrics.record_generation_success();
                info!(
                    duration_ms = duration.as_millis(),
                    "Content generation completed"
                );
                Ok(())
            }
            Err(e) => {
                self.metrics.record_generation_failure();
                error!(
                    duration_ms = duration.as_millis(),
                    error = ?e,
                    "Content generation failed"
                );
                Err(e.into())
            }
        }
    }
}
