use crate::{CurationBot, CurationMessage, GenerationBot, GenerationMessage, PostingBot, PostingMessage};
use botticelli_error::BotticelliResult;
use ractor::{Actor, ActorRef};
use std::time::Duration;
use tracing::{error, info, instrument};

/// Bot server that orchestrates generation, curation, and posting actors.
pub struct BotServer {
    generation_ref: Option<ActorRef<GenerationMessage>>,
    curation_ref: Option<ActorRef<CurationMessage>>,
    posting_ref: Option<ActorRef<PostingMessage>>,
}

impl BotServer {
    /// Creates a new bot server.
    pub fn new() -> Self {
        Self {
            generation_ref: None,
            curation_ref: None,
            posting_ref: None,
        }
    }

    /// Starts all bots with their respective intervals.
    #[instrument(skip(self))]
    pub async fn start(
        &mut self,
        generation_interval: Duration,
        curation_interval: Duration,
        posting_interval: Duration,
    ) -> BotticelliResult<()> {
        info!("Starting bot server");

        // Spawn generation bot
        let (generation_ref, _) = Actor::spawn(
            Some("generation_bot".to_string()),
            GenerationBot::new(generation_interval),
            generation_interval,
        )
        .await
        .map_err(|e| {
            error!(error = ?e, "Failed to spawn generation bot");
            botticelli_error::BotticelliError::new_server("Failed to spawn generation bot")
        })?;

        self.generation_ref = Some(generation_ref.clone());
        generation_ref
            .send_message(GenerationMessage::Start)
            .map_err(|e| {
                error!(error = ?e, "Failed to start generation bot");
                botticelli_error::BotticelliError::new_server("Failed to start generation bot")
            })?;

        // Spawn curation bot
        let (curation_ref, _) = Actor::spawn(
            Some("curation_bot".to_string()),
            CurationBot::new(curation_interval),
            curation_interval,
        )
        .await
        .map_err(|e| {
            error!(error = ?e, "Failed to spawn curation bot");
            botticelli_error::BotticelliError::new_server("Failed to spawn curation bot")
        })?;

        self.curation_ref = Some(curation_ref.clone());
        curation_ref
            .send_message(CurationMessage::Start)
            .map_err(|e| {
                error!(error = ?e, "Failed to start curation bot");
                botticelli_error::BotticelliError::new_server("Failed to start curation bot")
            })?;

        // Spawn posting bot
        let (posting_ref, _) = Actor::spawn(
            Some("posting_bot".to_string()),
            PostingBot::new(posting_interval),
            posting_interval,
        )
        .await
        .map_err(|e| {
            error!(error = ?e, "Failed to spawn posting bot");
            botticelli_error::BotticelliError::new_server("Failed to spawn posting bot")
        })?;

        self.posting_ref = Some(posting_ref.clone());
        posting_ref
            .send_message(PostingMessage::Start)
            .map_err(|e| {
                error!(error = ?e, "Failed to start posting bot");
                botticelli_error::BotticelliError::new_server("Failed to start posting bot")
            })?;

        info!("All bots started");
        Ok(())
    }

    /// Stops all bots.
    #[instrument(skip(self))]
    pub async fn stop(&mut self) -> BotticelliResult<()> {
        info!("Stopping bot server");

        if let Some(ref gen) = self.generation_ref {
            let _ = gen.send_message(GenerationMessage::Stop);
            gen.stop(None);
        }

        if let Some(ref cur) = self.curation_ref {
            let _ = cur.send_message(CurationMessage::Stop);
            cur.stop(None);
        }

        if let Some(ref post) = self.posting_ref {
            let _ = post.send_message(PostingMessage::Stop);
            post.stop(None);
        }

        self.generation_ref = None;
        self.curation_ref = None;
        self.posting_ref = None;

        info!("All bots stopped");
        Ok(())
    }

    /// Returns whether the server is running.
    pub fn is_running(&self) -> bool {
        self.generation_ref.is_some()
            || self.curation_ref.is_some()
            || self.posting_ref.is_some()
    }
}

impl Default for BotServer {
    fn default() -> Self {
        Self::new()
    }
}
