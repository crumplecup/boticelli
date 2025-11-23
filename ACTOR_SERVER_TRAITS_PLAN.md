# Actor Server Traits Implementation Plan

## Status: All Phases Complete ✅

All phases have been implemented and tested:
- **botticelli_server**: Actor server traits defined in `actor_traits.rs`
- **botticelli_actor**: Generic implementations in `server.rs`
- **botticelli_actor**: Discord-specific implementations in `discord_server.rs`
- **Tests**: Comprehensive test coverage in `tests/discord_server_test.rs`
- All checks passing (lint, format, tests, doctests)

## Overview

Extend `botticelli_server` with traits for hosting long-running actor-based services, then implement those traits specifically for `botticelli_actor`. This creates a flexible server framework that can host diverse service types while providing actor-specific implementations.

## Design Philosophy

**botticelli_server Purpose:** Host diverse server traits for different service types (LLM inference, bot orchestration, API gateways, etc.)

**Key Principle:** Traits define **what servers can do**, implementations define **how specific servers work**.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    botticelli_server                         │
│                   (Trait Definitions)                        │
│                                                               │
│  ┌──────────────────┐  ┌──────────────────┐                 │
│  │ InferenceServer  │  │  ScheduledServer │  (generic)      │
│  │   (existing)     │  │     (new)        │                 │
│  └──────────────────┘  └──────────────────┘                 │
│                                                               │
│  ┌──────────────────┐  ┌──────────────────┐                 │
│  │  StatefulServer  │  │   EventServer    │  (extensible)   │
│  │     (new)        │  │     (new)        │                 │
│  └──────────────────┘  └──────────────────┘                 │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ implements
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    botticelli_actor                          │
│               (Actor-Specific Implementation)                │
│                                                               │
│  ┌────────────────────────────────────────┐                 │
│  │         ActorScheduledServer           │                 │
│  │  impl ScheduledServer + StatefulServer │                 │
│  │                                         │                 │
│  │  - Uses Actor from same crate          │                 │
│  │  - Manages schedules and lifecycles    │                 │
│  │  - Persists execution history          │                 │
│  └────────────────────────────────────────┘                 │
└─────────────────────────────────────────────────────────────┘
```

## Implemented Design

### Phase 1: Actor Server Traits (botticelli_server) ✅

**Location**: `crates/botticelli_server/src/actor_traits.rs`

**Traits Defined**:
1. **TaskScheduler** - Schedule periodic tasks with tokio intervals
2. **ActorManager<ActorId, Context>** - Manage actor registration and execution
3. **ContentPoster<Content, Destination, Posted>** - Platform-agnostic posting
4. **StatePersistence<State>** - Save/load state across restarts
5. **ActorServer** - Main coordinator for starting/stopping/reloading

All traits use `async_trait` and return `ActorServerResult<T>` (type alias for `Result<T, Box<dyn Error + Send + Sync>>`).

### Phase 2: Generic Implementations (botticelli_actor) ✅

**Location**: `crates/botticelli_actor/src/server.rs`

**Types Implemented**:
1. **SimpleTaskScheduler** - In-memory scheduler using tokio spawn + interval
2. **GenericActorManager<I, C>** - Generic actor registry with HashMap
3. **GenericContentPoster<Content, Dest, Posted>** - Stub implementation
4. **JsonStatePersistence<T>** - JSON file-based persistence with serde
5. **BasicActorServer** - Minimal coordinator with running state

All implementations include full `#[instrument]` tracing.

## Usage Example

```rust
use botticelli_actor::{SimpleTaskScheduler, GenericActorManager, JsonStatePersistence, BasicActorServer};
use botticelli_server::{TaskScheduler, ActorManager, StatePersistence, ActorServer};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create components
    let mut scheduler = SimpleTaskScheduler::new();
    let mut manager = GenericActorManager::<String, ()>::new();
    let persistence = JsonStatePersistence::<Vec<String>>::new("state.json");
    let mut server = BasicActorServer::new();
    
    // Register an actor
    manager.register_actor("my_actor".to_string()).await?;
    
    // Schedule a task
    scheduler.schedule(
        "periodic_task".to_string(),
        Duration::from_secs(60),
        || async {
            println!("Task executing...");
            Ok(())
        }
    ).await?;
    
    // Save state
    let state = vec!["data1".to_string(), "data2".to_string()];
    persistence.save_state(&state).await?;
    
    // Start server
    server.start().await?;
    
    Ok(())
}
```

## Discord Integration Complete ✅

### Phase 3: Discord Actor Server (COMPLETE)

**Location**: `crates/botticelli_actor/src/discord_server.rs`

Implemented concrete Discord-specific actor server:

1. **DiscordActorId** - Actor identifier combining name + channel
2. **DiscordContext** - Execution context with HTTP client (no PgConnection due to !Sync)
3. **DiscordTaskScheduler** - Task scheduler implementation for Discord actors
4. **DiscordActorManager** - Manages Discord bot actors with serenity integration
5. **DiscordContentPoster** - Posts Content to Discord channels
6. **DiscordServerState** - Serializable state for persistence
7. **DiscordActorServer** - Complete server coordinating all components

**Key Design Decisions**:
- `PgConnection` excluded from `DiscordContext` because it's not `Sync`
- Database connections should be obtained from connection pool when needed
- Uses `ContentPost` to record posted content with platform IDs
- Token obtained from environment variable or config
- Full tracing instrumentation on all operations

**New Types Exported**:
- All Discord server types exported under `#[cfg(feature = "discord")]`
- `ContentPost` added to track posted content with metadata

### Phase 4: Testing & Documentation (COMPLETE) ✅

**Location**: `crates/botticelli_actor/tests/discord_server_test.rs`

Comprehensive test coverage includes:
1. **Discord Actor ID** - Creation and field access
2. **Discord Context** - HTTP client management
3. **Task Scheduler** - Scheduling and canceling tasks
4. **Actor Manager** - Manager creation
5. **Content Poster** - Poster initialization
6. **Server State** - JSON serialization/deserialization
7. **Content Post** - Post creation with builder pattern
8. **Actor Server** - Server creation and state management

All tests passing with proper async handling and cleanup.

1. Unit tests for Discord server implementations
2. Integration tests with mock serenity HTTP
3. Example Discord actor server
4. Connection pool integration for database access
5. Update DISCORD_CONTENT_ACTOR_PLAN.md with usage examples

## Original Phase 1 Design (Reference)

### Goal
Define generic traits for scheduled, stateful, and event-driven servers.

### Implementation Steps

#### Step 1.1: Create schedule module

**File:** `crates/botticelli_server/src/schedule.rs`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Defines when a task should execute.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ScheduleType {
    /// Execute based on cron expression.
    Cron { expression: String },
    
    /// Execute at fixed intervals.
    Interval { seconds: u64 },
    
    /// Execute once at a specific time.
    Once { at: DateTime<Utc> },
    
    /// Execute immediately on startup.
    Immediate,
}

/// Result of checking if a schedule should run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduleCheck {
    pub should_run: bool,
    pub next_run: DateTime<Utc>,
}

/// Trait for schedule evaluation.
pub trait Schedule {
    /// Check if the schedule should run now.
    fn check(&self, last_run: Option<DateTime<Utc>>) -> ScheduleCheck;
    
    /// Get the next scheduled execution time.
    fn next_execution(&self, after: DateTime<Utc>) -> DateTime<Utc>;
}
```

#### Step 1.2: Create scheduled server trait

**File:** `crates/botticelli_server/src/scheduled.rs`

```rust
use crate::schedule::{Schedule, ScheduleType};
use chrono::{DateTime, Utc};
use std::time::Duration;

/// Represents a task execution result.
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub task_id: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub success: bool,
    pub error_message: Option<String>,
    pub metadata: serde_json::Value,
}

/// Status of a scheduled task.
#[derive(Debug, Clone)]
pub struct TaskStatus {
    pub id: String,
    pub name: String,
    pub is_paused: bool,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: DateTime<Utc>,
    pub consecutive_failures: u32,
}

/// Trait for servers that execute tasks on schedules.
#[async_trait::async_trait]
pub trait ScheduledServer: Send + Sync {
    /// Execute a single scheduled task.
    async fn execute_task(&mut self, task_id: &str) -> Result<ExecutionResult, Box<dyn std::error::Error>>;
    
    /// Get the current status of a task.
    async fn task_status(&self, task_id: &str) -> Result<TaskStatus, Box<dyn std::error::Error>>;
    
    /// List all managed tasks.
    async fn list_tasks(&self) -> Result<Vec<TaskStatus>, Box<dyn std::error::Error>>;
    
    /// Pause a scheduled task.
    async fn pause_task(&mut self, task_id: &str) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Resume a paused task.
    async fn resume_task(&mut self, task_id: &str) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Main server loop - run forever.
    async fn run_forever(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Shutdown the server gracefully.
    async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}

/// Configuration for scheduled server behavior.
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// How often to check schedules (seconds).
    pub check_interval: Duration,
    
    /// Maximum consecutive failures before pausing.
    pub max_consecutive_failures: u32,
    
    /// Maximum parallel task executions.
    pub max_parallel_tasks: usize,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(60),
            max_consecutive_failures: 5,
            max_parallel_tasks: 1,
        }
    }
}
```

#### Step 1.3: Create stateful server trait

**File:** `crates/botticelli_server/src/stateful.rs`

```rust
use chrono::{DateTime, Utc};
use serde_json::Value;

/// Represents server state that persists across restarts.
#[derive(Debug, Clone)]
pub struct ServerState {
    pub task_id: String,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: DateTime<Utc>,
    pub consecutive_failures: u32,
    pub is_paused: bool,
    pub metadata: Value,
    pub updated_at: DateTime<Utc>,
}

/// Execution history record.
#[derive(Debug, Clone)]
pub struct ExecutionHistory {
    pub id: i64,
    pub task_id: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub success: bool,
    pub error_message: Option<String>,
    pub metadata: Value,
}

/// Trait for servers that persist state and execution history.
#[async_trait::async_trait]
pub trait StatefulServer: Send + Sync {
    /// Save current state for a task.
    async fn save_state(&self, state: &ServerState) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Load state for a task.
    async fn load_state(&self, task_id: &str) -> Result<Option<ServerState>, Box<dyn std::error::Error>>;
    
    /// Record an execution in history.
    async fn record_execution(&self, history: &ExecutionHistory) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Get recent execution history for a task.
    async fn get_history(
        &self,
        task_id: &str,
        limit: i64,
    ) -> Result<Vec<ExecutionHistory>, Box<dyn std::error::Error>>;
    
    /// Clear old execution history.
    async fn cleanup_history(&self, older_than: DateTime<Utc>) -> Result<u64, Box<dyn std::error::Error>>;
}
```

#### Step 1.4: Create event server trait (future-proofing)

**File:** `crates/botticelli_server/src/event.rs`

```rust
use serde_json::Value;

/// Event that can trigger server actions.
#[derive(Debug, Clone)]
pub struct ServerEvent {
    pub event_type: String,
    pub source: String,
    pub payload: Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Trait for servers that respond to events.
#[async_trait::async_trait]
pub trait EventServer: Send + Sync {
    /// Handle an incoming event.
    async fn handle_event(&mut self, event: ServerEvent) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Subscribe to event types.
    async fn subscribe(&mut self, event_types: Vec<String>) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Unsubscribe from event types.
    async fn unsubscribe(&mut self, event_types: Vec<String>) -> Result<(), Box<dyn std::error::Error>>;
}
```

#### Step 1.5: Update lib.rs

**File:** `crates/botticelli_server/src/lib.rs`

```rust
mod schedule;
mod scheduled;
mod stateful;
mod event;

pub use schedule::{Schedule, ScheduleCheck, ScheduleType};
pub use scheduled::{ExecutionResult, ScheduledServer, SchedulerConfig, TaskStatus};
pub use stateful::{ExecutionHistory, ServerState, StatefulServer};
pub use event::{EventServer, ServerEvent};

// Existing exports remain
mod inference;
pub use inference::{InferenceServer, ServerConfig, /* ... */};
```

### Dependencies to Add

```toml
[dependencies]
async-trait = "0.1"
chrono = { workspace = true }
serde = { workspace = true }
serde_json = "1.0"
```

### Tests

**File:** `crates/botticelli_server/tests/scheduled_trait_test.rs`

```rust
use botticelli_server::{ScheduledServer, TaskStatus};

#[tokio::test]
async fn test_scheduled_server_trait() {
    // Mock implementation to verify trait is usable
    struct MockScheduler;
    
    #[async_trait::async_trait]
    impl ScheduledServer for MockScheduler {
        async fn execute_task(&mut self, _task_id: &str) -> Result<ExecutionResult, Box<dyn std::error::Error>> {
            todo!()
        }
        
        // ... implement other methods
    }
}
```

## Phase 2: Actor Server Implementation (botticelli_actor)

### Goal
Implement server traits within the `botticelli_actor` crate for actor orchestration.

### Implementation Steps

#### Step 2.1: Update Cargo.toml

**File:** `crates/botticelli_actor/Cargo.toml`

Add new dependencies:

```toml
[dependencies]
# Existing dependencies...
botticelli_server = { path = "../botticelli_server" }
cron = "0.12"
```

#### Step 2.2: Create schedule implementation

**File:** `crates/botticelli_actor/src/schedule_impl.rs`

```rust
use botticelli_server::{Schedule, ScheduleCheck, ScheduleType};
use chrono::{DateTime, Duration, Utc};
use cron::Schedule as CronSchedule;
use std::str::FromStr;

/// Concrete implementation of Schedule trait.
pub struct ScheduleImpl {
    schedule_type: ScheduleType,
    cron_schedule: Option<CronSchedule>,
}

impl ScheduleImpl {
    pub fn new(schedule_type: ScheduleType) -> Result<Self, String> {
        let cron_schedule = match &schedule_type {
            ScheduleType::Cron { expression } => {
                Some(CronSchedule::from_str(expression)
                    .map_err(|e| format!("Invalid cron expression: {}", e))?)
            }
            _ => None,
        };
        
        Ok(Self {
            schedule_type,
            cron_schedule,
        })
    }
}

impl Schedule for ScheduleImpl {
    fn check(&self, last_run: Option<DateTime<Utc>>) -> ScheduleCheck {
        let now = Utc::now();
        
        match &self.schedule_type {
            ScheduleType::Immediate => ScheduleCheck {
                should_run: last_run.is_none(),
                next_run: now,
            },
            
            ScheduleType::Once { at } => ScheduleCheck {
                should_run: last_run.is_none() && *at <= now,
                next_run: *at,
            },
            
            ScheduleType::Interval { seconds } => {
                let next_run = last_run
                    .map(|lr| lr + Duration::seconds(*seconds as i64))
                    .unwrap_or(now);
                
                ScheduleCheck {
                    should_run: next_run <= now,
                    next_run,
                }
            },
            
            ScheduleType::Cron { .. } => {
                let schedule = self.cron_schedule.as_ref().unwrap();
                let next_run = schedule
                    .after(&last_run.unwrap_or(now))
                    .next()
                    .unwrap();
                
                ScheduleCheck {
                    should_run: next_run <= now,
                    next_run,
                }
            },
        }
    }
    
    fn next_execution(&self, after: DateTime<Utc>) -> DateTime<Utc> {
        self.check(Some(after)).next_run
    }
}
```

#### Step 2.3: Create actor server core

**File:** `crates/botticelli_actor/src/actor_server.rs`

```rust
use crate::Actor;
use botticelli_server::{
    ExecutionResult, ScheduledServer, SchedulerConfig, TaskStatus,
    ServerState, StatefulServer, ExecutionHistory,
};
use crate::schedule_impl::ScheduleImpl;
use chrono::Utc;
use diesel::PgConnection;
use std::collections::HashMap;
use tracing::{debug, error, info, instrument};

/// Scheduled task wrapping an Actor.
pub struct ScheduledActor {
    pub id: String,
    pub name: String,
    pub actor: Actor,
    pub schedule: ScheduleImpl,
    pub state: TaskState,
}

#[derive(Debug, Clone)]
pub struct TaskState {
    pub last_run: Option<chrono::DateTime<chrono::Utc>>,
    pub next_run: chrono::DateTime<chrono::Utc>,
    pub consecutive_failures: u32,
    pub is_paused: bool,
}

/// Actor-based implementation of ScheduledServer.
pub struct ActorScheduledServer {
    tasks: HashMap<String, ScheduledActor>,
    config: SchedulerConfig,
    database_url: String,
}

impl ActorScheduledServer {
    pub fn new(database_url: String, config: SchedulerConfig) -> Self {
        Self {
            tasks: HashMap::new(),
            config,
            database_url,
        }
    }
    
    pub fn add_task(
        &mut self,
        id: String,
        name: String,
        actor: Actor,
        schedule: ScheduleImpl,
    ) -> Result<(), String> {
        let state = TaskState {
            last_run: None,
            next_run: schedule.next_execution(Utc::now()),
            consecutive_failures: 0,
            is_paused: false,
        };
        
        let scheduled = ScheduledActor {
            id: id.clone(),
            name,
            actor,
            schedule,
            state,
        };
        
        self.tasks.insert(id, scheduled);
        Ok(())
    }
    
    #[instrument(skip(self, conn))]
    async fn execute_actor_internal(
        &mut self,
        task_id: &str,
        conn: &mut PgConnection,
    ) -> Result<ExecutionResult, Box<dyn std::error::Error>> {
        let task = self.tasks.get_mut(task_id)
            .ok_or_else(|| format!("Task not found: {}", task_id))?;
        
        info!(task_name = %task.name, "Executing scheduled actor");
        
        let started_at = Utc::now();
        
        let result = task.actor.execute(conn).await;
        
        let completed_at = Utc::now();
        
        let (success, error_message, metadata) = match result {
            Ok(exec) => {
                task.state.consecutive_failures = 0;
                task.state.last_run = Some(completed_at);
                task.state.next_run = task.schedule.next_execution(completed_at);
                
                info!(
                    succeeded = exec.succeeded.len(),
                    failed = exec.failed.len(),
                    "Actor execution completed"
                );
                
                (
                    exec.failed.is_empty(),
                    None,
                    serde_json::json!({
                        "succeeded": exec.succeeded,
                        "failed": exec.failed,
                        "skipped": exec.skipped,
                    }),
                )
            }
            Err(e) => {
                task.state.consecutive_failures += 1;
                
                if task.state.consecutive_failures >= self.config.max_consecutive_failures {
                    error!(
                        failures = task.state.consecutive_failures,
                        "Max failures reached, pausing task"
                    );
                    task.state.is_paused = true;
                }
                
                error!(error = ?e, "Actor execution failed");
                
                (false, Some(e.to_string()), serde_json::json!({}))
            }
        };
        
        Ok(ExecutionResult {
            task_id: task_id.to_string(),
            started_at,
            completed_at,
            success,
            error_message,
            metadata,
        })
    }
}

#[async_trait::async_trait]
impl ScheduledServer for ActorScheduledServer {
    async fn execute_task(&mut self, task_id: &str) -> Result<ExecutionResult, Box<dyn std::error::Error>> {
        // Get database connection
        let mut conn = diesel::pg::PgConnection::establish(&self.database_url)?;
        
        self.execute_actor_internal(task_id, &mut conn).await
    }
    
    async fn task_status(&self, task_id: &str) -> Result<TaskStatus, Box<dyn std::error::Error>> {
        let task = self.tasks.get(task_id)
            .ok_or_else(|| format!("Task not found: {}", task_id))?;
        
        Ok(TaskStatus {
            id: task.id.clone(),
            name: task.name.clone(),
            is_paused: task.state.is_paused,
            last_run: task.state.last_run,
            next_run: task.state.next_run,
            consecutive_failures: task.state.consecutive_failures,
        })
    }
    
    async fn list_tasks(&self) -> Result<Vec<TaskStatus>, Box<dyn std::error::Error>> {
        let mut statuses = Vec::new();
        for task_id in self.tasks.keys() {
            statuses.push(self.task_status(task_id).await?);
        }
        Ok(statuses)
    }
    
    async fn pause_task(&mut self, task_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let task = self.tasks.get_mut(task_id)
            .ok_or_else(|| format!("Task not found: {}", task_id))?;
        
        task.state.is_paused = true;
        info!(task_id, "Task paused");
        Ok(())
    }
    
    async fn resume_task(&mut self, task_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let task = self.tasks.get_mut(task_id)
            .ok_or_else(|| format!("Task not found: {}", task_id))?;
        
        task.state.is_paused = false;
        task.state.consecutive_failures = 0;
        info!(task_id, "Task resumed");
        Ok(())
    }
    
    async fn run_forever(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting actor scheduler");
        
        loop {
            debug!("Checking scheduled tasks");
            
            let task_ids: Vec<String> = self.tasks.keys().cloned().collect();
            
            for task_id in task_ids {
                let should_run = {
                    let task = self.tasks.get(&task_id).unwrap();
                    !task.state.is_paused && task.schedule.check(task.state.last_run).should_run
                };
                
                if should_run {
                    match self.execute_task(&task_id).await {
                        Ok(result) => {
                            info!(
                                task_id,
                                success = result.success,
                                "Task execution completed"
                            );
                        }
                        Err(e) => {
                            error!(task_id, error = ?e, "Task execution error");
                        }
                    }
                }
            }
            
            tokio::time::sleep(self.config.check_interval).await;
        }
    }
    
    async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Shutting down actor scheduler");
        Ok(())
    }
}
```

#### Step 2.4: Update lib.rs

**File:** `crates/botticelli_actor/src/lib.rs`

Add new modules and exports:

```rust
mod schedule_impl;
mod actor_server;

pub use schedule_impl::ScheduleImpl;
pub use actor_server::{ActorScheduledServer, ScheduledActor, TaskState};
```

### Tests

**File:** `crates/botticelli_actor/tests/actor_server_test.rs`

```rust
use botticelli_actor::{ActorScheduledServer, ScheduleImpl};
use botticelli_server::{ScheduleType, ScheduledServer, SchedulerConfig};

#[tokio::test]
async fn test_actor_server_creation() {
    let config = SchedulerConfig::default();
    let server = ActorScheduledServer::new(
        "postgresql://localhost/test".to_string(),
        config,
    );
    
    assert!(server.list_tasks().await.unwrap().is_empty());
}
```

## Phase 3: Database Schema for State Persistence

### Implementation Steps

#### Step 3.1: Create migration

**File:** `migrations/YYYY-MM-DD-HHMMSS_create_actor_server_tables/up.sql`

```sql
-- Server state tracking
CREATE TABLE actor_server_state (
    task_id VARCHAR(255) PRIMARY KEY,
    last_run TIMESTAMPTZ,
    next_run TIMESTAMPTZ NOT NULL,
    consecutive_failures INTEGER DEFAULT 0,
    is_paused BOOLEAN DEFAULT FALSE,
    metadata JSONB DEFAULT '{}',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Execution history
CREATE TABLE actor_server_executions (
    id SERIAL PRIMARY KEY,
    task_id VARCHAR(255) NOT NULL,
    started_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ,
    success BOOLEAN DEFAULT FALSE,
    error_message TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_actor_server_executions_task ON actor_server_executions(task_id);
CREATE INDEX idx_actor_server_executions_started ON actor_server_executions(started_at);
CREATE INDEX idx_actor_server_state_next_run ON actor_server_state(next_run) WHERE NOT is_paused;
```

**File:** `migrations/YYYY-MM-DD-HHMMSS_create_actor_server_tables/down.sql`

```sql
DROP TABLE actor_server_executions;
DROP TABLE actor_server_state;
```

#### Step 3.2: Implement StatefulServer trait

**File:** `crates/botticelli_actor/src/state_store.rs`

```rust
use botticelli_server::{ExecutionHistory, ServerState, StatefulServer};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use tracing::instrument;

/// Database-backed state persistence.
pub struct StateStore {
    database_url: String,
}

impl StateStore {
    pub fn new(database_url: String) -> Self {
        Self { database_url }
    }
}

#[async_trait::async_trait]
impl StatefulServer for StateStore {
    #[instrument(skip(self, state))]
    async fn save_state(&self, state: &ServerState) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement using Diesel
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn load_state(&self, task_id: &str) -> Result<Option<ServerState>, Box<dyn std::error::Error>> {
        // TODO: Implement using Diesel
        Ok(None)
    }
    
    #[instrument(skip(self, history))]
    async fn record_execution(&self, history: &ExecutionHistory) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement using Diesel
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn get_history(
        &self,
        task_id: &str,
        limit: i64,
    ) -> Result<Vec<ExecutionHistory>, Box<dyn std::error::Error>> {
        // TODO: Implement using Diesel
        Ok(Vec::new())
    }
    
    #[instrument(skip(self))]
    async fn cleanup_history(&self, older_than: DateTime<Utc>) -> Result<u64, Box<dyn std::error::Error>> {
        // TODO: Implement using Diesel
        Ok(0)
    }
}
```

## Phase 4: Binary and Configuration

### Implementation Steps

#### Step 4.1: Create binary crate structure

```bash
mkdir -p crates/botticelli_actor/src/bin
```

**File:** `crates/botticelli_actor/src/bin/actor-server.rs`

```rust
use botticelli_actor::{ActorScheduledServer, ScheduleImpl};
use botticelli_server::{ScheduleType, ScheduledServer, SchedulerConfig};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    info!("Starting botticelli actor server");
    
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    let config = SchedulerConfig::default();
    let mut server = ActorScheduledServer::new(database_url, config);
    
    // TODO: Load actors from configuration
    
    server.run_forever().await?;
    
    Ok(())
}
```

#### Step 4.2: Configuration file format

**File:** `examples/actor_server.toml`

```toml
[server]
database_url = "postgresql://localhost/botticelli"
check_interval_seconds = 60
max_consecutive_failures = 5
max_parallel_tasks = 1

[[tasks]]
id = "daily_poster"
name = "Daily Content Poster"
actor_config = "actors/daily_poster.toml"

[tasks.schedule]
type = "Cron"
expression = "0 9 * * *"  # 9 AM daily

[[tasks]]
id = "hourly_monitor"
name = "Trending Monitor"
actor_config = "actors/trending.toml"

[tasks.schedule]
type = "Interval"
seconds = 3600
```

## Phase 5: HTTP API (Optional, Future)

### Implementation Steps

Add REST API for server control using axum:

- GET `/health` - Health check
- GET `/tasks` - List all tasks
- GET `/tasks/:id` - Get task status
- POST `/tasks/:id/trigger` - Manually trigger
- POST `/tasks/:id/pause` - Pause task
- POST `/tasks/:id/resume` - Resume task
- GET `/tasks/:id/history` - Execution history

## Verification Checklist

### Phase 1: Traits
- [ ] `ScheduledServer` trait compiles
- [ ] `StatefulServer` trait compiles
- [ ] `EventServer` trait compiles
- [ ] Trait methods have appropriate signatures
- [ ] Documentation complete

### Phase 2: Implementation
- [ ] `ActorScheduledServer` implements `ScheduledServer`
- [ ] `ScheduleImpl` works for all schedule types
- [ ] Tasks can be added and listed
- [ ] Tasks can be paused and resumed
- [ ] Execution loop runs without errors
- [ ] All instrumentation in place

### Phase 3: State
- [ ] Migrations run successfully
- [ ] `StateStore` implements `StatefulServer`
- [ ] State persists across restarts
- [ ] Execution history recorded
- [ ] Old history can be cleaned

### Phase 4: Binary
- [ ] Binary compiles and runs
- [ ] Configuration loaded correctly
- [ ] Actors loaded from config
- [ ] Server starts and schedules tasks
- [ ] Graceful shutdown works

## Next Steps

1. Implement Phase 1 (server traits in botticelli_server)
2. Run `just check-all botticelli_server`
3. Implement Phase 2 (actor server in botticelli_actor)
4. Run `just check-all botticelli_actor`
5. Implement Phase 3 (database)
6. Run full integration test
7. Document usage in SERVER_GUIDE.md

## Open Questions

1. **Should StateStore be separate from ActorScheduledServer?**
   - Pro: Clean separation
   - Con: More complexity, two traits to implement
   - **Decision:** Keep separate, allows flexible backends (Redis, file, etc.)

2. **How to handle actor configuration loading?**
   - Load from TOML files referenced in server config
   - Need to instantiate Platform implementations
   - **Decision:** Factory pattern for platform creation

3. **Parallel execution strategy?**
   - Sequential is safe but slow
   - Parallel requires careful resource management
   - **Decision:** Start sequential, add parallelism with bounded task pool

4. **Error recovery policy?**
   - Circuit breaker (pause after N failures)
   - Exponential backoff
   - Alert on repeated failures
   - **Decision:** Circuit breaker + configurable threshold
