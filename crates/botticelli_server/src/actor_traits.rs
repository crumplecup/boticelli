//! Trait interfaces for actor-based bot server implementations
//!
//! This module defines traits for long-running bot servers that execute
//! periodic tasks using actors with skills and knowledge bases.

use async_trait::async_trait;
use std::time::Duration;

/// Result type for actor server operations
pub type ActorServerResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Trait for scheduling and executing periodic tasks
///
/// Implementations manage task execution timing and coordination.
#[async_trait]
pub trait TaskScheduler: Send + Sync {
    /// Schedule a task to run periodically
    ///
    /// # Parameters
    /// - `task_id`: Unique identifier for the task
    /// - `interval`: Duration between task executions
    /// - `task`: Async function to execute
    async fn schedule<F, Fut>(
        &mut self,
        task_id: String,
        interval: Duration,
        task: F,
    ) -> ActorServerResult<()>
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ActorServerResult<()>> + Send + 'static;

    /// Cancel a scheduled task
    async fn cancel(&mut self, task_id: &str) -> ActorServerResult<()>;

    /// Check if a task is currently scheduled
    fn is_scheduled(&self, task_id: &str) -> bool;

    /// Get all scheduled task IDs
    fn scheduled_tasks(&self) -> Vec<String>;
}

/// Trait for managing actor lifecycle and execution
///
/// Implementations coordinate actors, their skills, and knowledge bases.
#[async_trait]
pub trait ActorManager: Send + Sync {
    /// Type representing an actor's unique identifier
    type ActorId: Send + Sync + Clone;

    /// Type representing execution context (platform-specific state)
    type Context: Send + Sync;

    /// Register an actor with the manager
    async fn register_actor(&mut self, actor_id: Self::ActorId) -> ActorServerResult<()>;

    /// Unregister an actor
    async fn unregister_actor(&mut self, actor_id: &Self::ActorId) -> ActorServerResult<()>;

    /// Execute an actor's task with given context
    ///
    /// This triggers the actor to perform its configured behavior using
    /// available skills and knowledge.
    async fn execute_actor(
        &self,
        actor_id: &Self::ActorId,
        context: &Self::Context,
    ) -> ActorServerResult<()>;

    /// Get list of registered actor IDs
    fn registered_actors(&self) -> Vec<Self::ActorId>;

    /// Check if an actor is registered
    fn is_registered(&self, actor_id: &Self::ActorId) -> bool;
}

/// Trait for platform-specific posting operations
///
/// Implementations handle posting content to specific platforms (Discord, Twitter, etc.).
#[async_trait]
pub trait ContentPoster: Send + Sync {
    /// Type representing platform-specific content
    type Content: Send + Sync;

    /// Type representing platform-specific destination (channel, thread, etc.)
    type Destination: Send + Sync;

    /// Type representing posted content (with ID, timestamp, etc.)
    type Posted: Send + Sync;

    /// Post content to a destination
    async fn post(
        &self,
        content: Self::Content,
        destination: &Self::Destination,
    ) -> ActorServerResult<Self::Posted>;

    /// Check if posting is currently available (rate limits, connectivity, etc.)
    async fn can_post(&self, destination: &Self::Destination) -> bool;
}

/// Trait for state persistence across server restarts
///
/// Implementations save and restore actor state, schedules, and execution history.
#[async_trait]
pub trait StatePersistence: Send + Sync {
    /// Type representing serializable state
    type State: Send + Sync;

    /// Save current state to persistent storage
    async fn save_state(&self, state: &Self::State) -> ActorServerResult<()>;

    /// Load state from persistent storage
    async fn load_state(&self) -> ActorServerResult<Option<Self::State>>;

    /// Clear all persisted state
    async fn clear_state(&self) -> ActorServerResult<()>;
}

/// Trait for the main actor server that coordinates all components
///
/// Implementations combine task scheduling, actor management, posting, and persistence
/// into a unified long-running server.
#[async_trait]
pub trait ActorServer: Send + Sync {
    /// Start the server and begin processing scheduled tasks
    async fn start(&mut self) -> ActorServerResult<()>;

    /// Stop the server gracefully, finishing current tasks
    async fn stop(&mut self) -> ActorServerResult<()>;

    /// Check if the server is currently running
    fn is_running(&self) -> bool;

    /// Reload configuration and restart affected actors
    async fn reload(&mut self) -> ActorServerResult<()>;
}
