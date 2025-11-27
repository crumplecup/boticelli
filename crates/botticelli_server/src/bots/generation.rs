use ractor::{Actor, ActorProcessingErr, ActorRef};
use std::time::Duration;
use tracing::{debug, error, info, instrument};

/// Messages for the GenerationBot actor
#[derive(Debug, Clone)]
pub enum GenerationMessage {
    /// Start the generation loop
    Start,
    /// Stop the generation loop
    Stop,
    /// Run one generation cycle
    RunCycle,
}

/// Bot that generates content on a schedule
pub struct GenerationBot {
    running: bool,
    interval: Duration,
}

impl GenerationBot {
    /// Creates a new generation bot
    pub fn new(interval: Duration) -> Self {
        Self {
            running: false,
            interval,
        }
    }

    #[instrument(skip(self))]
    async fn run_generation_cycle(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Running generation cycle");
        
        // TODO: Execute generation narrative
        // For now, just log
        debug!(interval_secs = ?self.interval.as_secs(), "Generation cycle placeholder");
        
        Ok(())
    }
}

#[async_trait::async_trait]
impl Actor for GenerationBot {
    type Msg = GenerationMessage;
    type State = ();
    type Arguments = Duration;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        interval: Duration,
    ) -> Result<Self::State, ActorProcessingErr> {
        info!(interval_secs = ?interval.as_secs(), "GenerationBot starting");
        Ok(())
    }

    #[instrument(skip(self, _myself, _state))]
    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        _state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            GenerationMessage::Start => {
                info!("Starting generation loop");
                // TODO: Spawn background task that runs every interval
            }
            GenerationMessage::Stop => {
                info!("Stopping generation loop");
            }
            GenerationMessage::RunCycle => {
                if let Err(e) = self.run_generation_cycle().await {
                    error!(error = ?e, "Generation cycle failed");
                }
            }
        }
        Ok(())
    }
}
