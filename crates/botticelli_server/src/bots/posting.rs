use ractor::{Actor, ActorProcessingErr, ActorRef};
use rand::Rng;
use std::time::Duration;
use tracing::{debug, error, info, instrument};

/// Messages for the PostingBot actor
#[derive(Debug, Clone)]
pub enum PostingMessage {
    /// Start the posting loop
    Start,
    /// Stop the posting loop
    Stop,
    /// Post one piece of approved content
    PostNext,
}

/// Bot that posts curated content to Discord
pub struct PostingBot {
    running: bool,
    base_interval: Duration,
    jitter_percent: f64,
}

impl PostingBot {
    /// Creates a new posting bot
    pub fn new(base_interval: Duration, jitter_percent: f64) -> Self {
        Self {
            running: false,
            base_interval,
            jitter_percent,
        }
    }

    /// Calculate next posting delay with jitter
    #[instrument(skip(self))]
    fn calculate_next_delay(&self) -> Duration {
        let mut rng = rand::thread_rng();
        let jitter_range = (self.base_interval.as_secs_f64() * self.jitter_percent) as i64;
        let jitter = rng.gen_range(-jitter_range..=jitter_range);
        let next_secs = (self.base_interval.as_secs() as i64 + jitter).max(60); // Minimum 1 minute
        
        let delay = Duration::from_secs(next_secs as u64);
        debug!(
            base_secs = self.base_interval.as_secs(),
            jitter_secs = jitter,
            next_secs,
            "Calculated next posting delay"
        );
        delay
    }

    #[instrument(skip(self))]
    async fn post_next_content(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Posting next approved content");
        
        // TODO: Query approved_discord_posts for oldest unposted content
        // TODO: Execute posting narrative or direct Discord API call
        // TODO: Mark content as posted with timestamp
        
        debug!("Content posted successfully");
        Ok(())
    }
}

#[async_trait::async_trait]
impl Actor for PostingBot {
    type Msg = PostingMessage;
    type State = ();
    type Arguments = (Duration, f64);

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        (base_interval, jitter_percent): (Duration, f64),
    ) -> Result<Self::State, ActorProcessingErr> {
        info!(
            interval_hours = ?base_interval.as_secs() / 3600,
            jitter_percent,
            "PostingBot starting"
        );
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
            PostingMessage::Start => {
                info!("Starting posting loop");
                // TODO: Spawn background task with jittered intervals
            }
            PostingMessage::Stop => {
                info!("Stopping posting loop");
            }
            PostingMessage::PostNext => {
                if let Err(e) = self.post_next_content().await {
                    error!(error = ?e, "Posting failed");
                }
            }
        }
        Ok(())
    }
}
