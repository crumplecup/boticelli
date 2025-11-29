//! Discord-specific actor server implementations

use crate::{Actor, ActorConfig, Content, ContentPost, ContentPostBuilder, DiscordPlatform};
use async_trait::async_trait;
use botticelli_server::{
    ActorManager, ActorServer, ActorServerResult, ContentPoster, TaskScheduler,
};
use serenity::all::{ChannelId, CreateMessage, Http};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, instrument};

/// Discord actor identifier combining actor name and target channel
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct DiscordActorId {
    actor_name: String,
    channel_id: ChannelId,
}

impl DiscordActorId {
    /// Create a new Discord actor ID
    pub fn new(actor_name: impl Into<String>, channel_id: ChannelId) -> Self {
        Self {
            actor_name: actor_name.into(),
            channel_id,
        }
    }

    /// Get the actor name
    pub fn actor_name(&self) -> &str {
        &self.actor_name
    }

    /// Get the channel ID
    pub fn channel_id(&self) -> ChannelId {
        self.channel_id
    }
}

/// Discord execution context containing HTTP client
///
/// Note: Database connection is not included because PgConnection is not Sync.
/// Connections should be obtained from a connection pool when needed.
#[derive(Clone)]
pub struct DiscordContext {
    http: Arc<Http>,
}

impl DiscordContext {
    /// Create a new Discord context
    pub fn new(http: Arc<Http>) -> Self {
        Self { http }
    }

    /// Get the HTTP client
    pub fn http(&self) -> &Arc<Http> {
        &self.http
    }
}

/// Task scheduler implementation for Discord actors
#[derive(Debug)]
pub struct DiscordTaskScheduler {
    tasks: Arc<RwLock<HashMap<DiscordActorId, JoinHandle<()>>>>,
}

impl DiscordTaskScheduler {
    /// Create a new Discord task scheduler
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for DiscordTaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TaskScheduler for DiscordTaskScheduler {
    #[instrument(skip(self, task))]
    async fn schedule<F, Fut>(
        &mut self,
        task_id: String,
        interval: Duration,
        task: F,
    ) -> ActorServerResult<()>
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ActorServerResult<()>> + Send + 'static,
    {
        debug!(?interval, "Scheduling Discord actor task");

        // Parse task_id back into DiscordActorId
        let parts: Vec<&str> = task_id.split(':').collect();
        if parts.len() != 2 {
            return Err("Invalid task_id format".into());
        }
        let actor_name = parts[0].to_string();
        let channel_id = ChannelId::new(parts[1].parse()?);
        let actor_id = DiscordActorId::new(actor_name, channel_id);

        let task = Arc::new(task);
        let handle = tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            loop {
                interval_timer.tick().await;
                if let Err(e) = task().await {
                    error!(error = ?e, "Discord task execution failed");
                }
            }
        });

        let mut tasks = self.tasks.write().await;
        if let Some(old_handle) = tasks.insert(actor_id.clone(), handle) {
            debug!("Canceling existing Discord task");
            old_handle.abort();
        }

        info!(actor = %actor_id.actor_name, channel = %actor_id.channel_id, "Discord task scheduled");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn cancel(&mut self, task_id: &str) -> ActorServerResult<()> {
        debug!("Canceling Discord task");

        let parts: Vec<&str> = task_id.split(':').collect();
        if parts.len() != 2 {
            return Err("Invalid task_id format".into());
        }
        let actor_name = parts[0].to_string();
        let channel_id = ChannelId::new(parts[1].parse()?);
        let actor_id = DiscordActorId::new(actor_name, channel_id);

        let mut tasks = self.tasks.write().await;
        if let Some(handle) = tasks.remove(&actor_id) {
            handle.abort();
            info!("Discord task canceled");
        }
        Ok(())
    }

    fn is_scheduled(&self, task_id: &str) -> bool {
        let parts: Vec<&str> = task_id.split(':').collect();
        if parts.len() != 2 {
            return false;
        }
        let Ok(channel_id) = parts[1].parse::<u64>() else {
            return false;
        };
        let actor_id = DiscordActorId::new(parts[0], ChannelId::new(channel_id));

        let tasks = self.tasks.blocking_read();
        tasks.contains_key(&actor_id)
    }

    fn scheduled_tasks(&self) -> Vec<String> {
        let tasks = self.tasks.blocking_read();
        tasks
            .keys()
            .map(|id| format!("{}:{}", id.actor_name, id.channel_id.get()))
            .collect()
    }
}

/// Discord actor manager
pub struct DiscordActorManager {
    actors: Arc<RwLock<HashMap<DiscordActorId, Actor>>>,
    configs: Arc<RwLock<HashMap<DiscordActorId, ActorConfig>>>,
}

impl DiscordActorManager {
    /// Create a new Discord actor manager
    pub fn new() -> Self {
        Self {
            actors: Arc::new(RwLock::new(HashMap::new())),
            configs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add an actor configuration
    #[instrument(skip(self, config))]
    pub async fn add_actor_config(
        &mut self,
        actor_id: DiscordActorId,
        config: ActorConfig,
    ) -> ActorServerResult<()> {
        debug!(actor = %actor_id.actor_name, "Adding actor config");
        let mut configs = self.configs.write().await;
        configs.insert(actor_id, config);
        info!("Actor config added");
        Ok(())
    }
}

impl Default for DiscordActorManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ActorManager for DiscordActorManager {
    type ActorId = DiscordActorId;
    type Context = DiscordContext;

    #[instrument(skip(self))]
    async fn register_actor(&mut self, actor_id: Self::ActorId) -> ActorServerResult<()> {
        debug!(actor = %actor_id.actor_name, channel = %actor_id.channel_id, "Registering Discord actor");

        // Get config
        let configs = self.configs.read().await;
        let config = configs
            .get(&actor_id)
            .ok_or("Actor config not found")?
            .clone();
        drop(configs);

        // Create actor with builder pattern
        // TODO: Get token from environment or config for authentication
        let _token = std::env::var("DISCORD_TOKEN").unwrap_or_else(|_| "dummy_token".to_string());

        let platform = Arc::new(DiscordPlatform::new(actor_id.channel_id.to_string())?);
        let skills = crate::SkillRegistry::new();

        let actor = Actor::builder()
            .config(config)
            .platform(platform)
            .skills(skills)
            .build()?;

        let mut actors = self.actors.write().await;
        actors.insert(actor_id.clone(), actor);

        info!(actor = %actor_id.actor_name, "Discord actor registered");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn unregister_actor(&mut self, actor_id: &Self::ActorId) -> ActorServerResult<()> {
        debug!(actor = %actor_id.actor_name, "Unregistering Discord actor");
        let mut actors = self.actors.write().await;
        actors.remove(actor_id);
        info!("Discord actor unregistered");
        Ok(())
    }

    #[instrument(skip(self, _context))]
    async fn execute_actor(
        &self,
        actor_id: &Self::ActorId,
        _context: &Self::Context,
    ) -> ActorServerResult<()> {
        debug!(actor = %actor_id.actor_name, "Executing Discord actor");

        let actors = self.actors.read().await;
        let _actor = actors.get(actor_id).ok_or("Actor not registered")?;

        // Execute actor logic - requires mutable connection
        // This is a simplified version; actual implementation would need
        // to properly handle database connection passing
        info!(actor = %actor_id.actor_name, "Discord actor execution skipped (needs DB connection)");

        // TODO: Implement proper execution with database connection
        // actor.execute(conn).await?;

        Ok(())
    }

    fn registered_actors(&self) -> Vec<Self::ActorId> {
        let actors = self.actors.blocking_read();
        actors.keys().cloned().collect()
    }

    fn is_registered(&self, actor_id: &Self::ActorId) -> bool {
        let actors = self.actors.blocking_read();
        actors.contains_key(actor_id)
    }
}

/// Discord content poster
pub struct DiscordContentPoster {
    http: Arc<Http>,
}

impl DiscordContentPoster {
    /// Create a new Discord content poster
    pub fn new(http: Arc<Http>) -> Self {
        Self { http }
    }
}

#[async_trait]
impl ContentPoster for DiscordContentPoster {
    type Content = Content;
    type Destination = ChannelId;
    type Posted = ContentPost;

    #[instrument(skip(self, content))]
    async fn post(
        &self,
        content: Self::Content,
        destination: &Self::Destination,
    ) -> ActorServerResult<Self::Posted> {
        debug!(channel = %destination, "Posting content to Discord");

        // Convert Content to Discord message
        let message_text = content.to_string();
        let message = CreateMessage::new().content(message_text);

        // Post to Discord
        let sent_message = destination.send_message(&self.http, message).await?;

        let post = ContentPostBuilder::default()
            .post_id(sent_message.id.to_string())
            .content(content)
            .destination(destination.to_string())
            .build()
            .map_err(|e| format!("Failed to build ContentPost: {}", e))?;

        info!(message_id = %sent_message.id, "Content posted to Discord");
        Ok(post)
    }

    async fn can_post(&self, _destination: &Self::Destination) -> bool {
        // Could check rate limits, permissions, etc.
        true
    }
}

/// Server state for persistence
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DiscordServerState {
    /// Last execution times for actors
    pub last_executions: HashMap<String, chrono::DateTime<chrono::Utc>>,
    /// Execution counts
    pub execution_counts: HashMap<String, u64>,
}

/// Discord actor server coordinating all components
pub struct DiscordActorServer {
    scheduler: DiscordTaskScheduler,
    manager: DiscordActorManager,
    poster: DiscordContentPoster,
    state_path: std::path::PathBuf,
    running: Arc<RwLock<bool>>,
}

impl DiscordActorServer {
    /// Create a new Discord actor server
    pub fn new(http: Arc<Http>, state_path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            scheduler: DiscordTaskScheduler::new(),
            manager: DiscordActorManager::new(),
            poster: DiscordContentPoster::new(http),
            state_path: state_path.into(),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Get mutable reference to the actor manager
    pub fn manager_mut(&mut self) -> &mut DiscordActorManager {
        &mut self.manager
    }

    /// Get mutable reference to the task scheduler
    pub fn scheduler_mut(&mut self) -> &mut DiscordTaskScheduler {
        &mut self.scheduler
    }

    /// Get reference to the content poster
    pub fn poster(&self) -> &DiscordContentPoster {
        &self.poster
    }

    /// Load state from disk
    #[instrument(skip(self))]
    async fn load_state(&self) -> ActorServerResult<DiscordServerState> {
        debug!(path = ?self.state_path, "Loading server state");
        if !self.state_path.exists() {
            info!("No existing state found, using defaults");
            return Ok(DiscordServerState::default());
        }

        let json = tokio::fs::read_to_string(&self.state_path).await?;
        let state = serde_json::from_str(&json)?;
        info!("Server state loaded");
        Ok(state)
    }

    /// Save state to disk
    #[instrument(skip(self, state))]
    async fn save_state(&self, state: &DiscordServerState) -> ActorServerResult<()> {
        debug!(path = ?self.state_path, "Saving server state");
        let json = serde_json::to_string_pretty(state)?;
        tokio::fs::write(&self.state_path, json).await?;
        info!("Server state saved");
        Ok(())
    }
}

#[async_trait]
impl ActorServer for DiscordActorServer {
    #[instrument(skip(self))]
    async fn start(&mut self) -> ActorServerResult<()> {
        info!("Starting Discord actor server");

        // Load state
        let _state = self.load_state().await?;

        let mut running = self.running.write().await;
        *running = true;

        info!("Discord actor server started");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn stop(&mut self) -> ActorServerResult<()> {
        info!("Stopping Discord actor server");

        // Save state before stopping
        let state = DiscordServerState::default();
        self.save_state(&state).await?;

        // Cancel all tasks
        let task_ids = self.scheduler.scheduled_tasks();
        for task_id in task_ids {
            self.scheduler.cancel(&task_id).await?;
        }

        let mut running = self.running.write().await;
        *running = false;

        info!("Discord actor server stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        *self.running.blocking_read()
    }

    #[instrument(skip(self))]
    async fn reload(&mut self) -> ActorServerResult<()> {
        info!("Reloading Discord actor server");
        // Could reload configs, restart actors, etc.
        Ok(())
    }
}
