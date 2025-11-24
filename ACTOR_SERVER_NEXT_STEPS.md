# Actor Server Next Steps Strategy

**Date**: 2025-11-23
**Status**: Foundational Implementation Complete, Productionization Pending

## Executive Summary

The actor server framework has successfully achieved **Phase 1 & 2 completion** with basic traits and Discord integration. The next phase focuses on **production readiness**: persistent state management, sophisticated scheduling, and operational tooling.

### What We Have (✅ Complete)

1. **Core Trait Framework** (`botticelli_server/src/actor_traits.rs`)
   - `TaskScheduler` - Periodic task execution
   - `ActorManager<ActorId, Context>` - Actor lifecycle management
   - `ContentPoster<Content, Dest, Posted>` - Platform-agnostic posting
   - `StatePersistence<State>` - State save/load interface
   - `ActorServer` - Coordinator for start/stop/reload

2. **Generic Implementations** (`botticelli_actor/src/server.rs`)
   - `SimpleTaskScheduler` - In-memory tokio-based scheduler
   - `GenericActorManager<I, C>` - HashMap-based actor registry
   - `GenericContentPoster` - Stub implementation
   - `JsonStatePersistence<T>` - JSON file persistence
   - `BasicActorServer` - Minimal running state coordinator

3. **Discord Integration** (`botticelli_actor/src/discord_server.rs`)
   - `DiscordActorId` - Actor identification
   - `DiscordContext` - HTTP client context
   - `DiscordTaskScheduler` - Discord-specific scheduling
   - `DiscordActorManager` - Actor management
   - `DiscordContentPoster` - serenity HTTP posting
   - `DiscordServerState` - Serializable state
   - `DiscordActorServer` - Full Discord server coordinator

4. **Test Coverage**
   - 14 passing tests (9 discord_server, 5 platform_trait)
   - Builder pattern compliance (per CLAUDE.md)
   - Full tracing instrumentation

### What We Don't Have (❌ Pending)

1. **Advanced Scheduling**
   - ❌ Cron expression support
   - ❌ `ScheduleType` enum (Cron, Interval, Once, Immediate)
   - ❌ `ScheduledServer` trait with task status/pause/resume
   - ❌ Schedule evaluation logic with next_run calculations

2. **Persistent State Management**
   - ❌ Database schema for actor_server_state table
   - ❌ Database schema for actor_server_executions table
   - ❌ `StatefulServer` trait implementation using Diesel
   - ❌ Execution history tracking
   - ❌ State recovery on server restart

3. **Production Operations**
   - ❌ Binary crate (`actor-server` executable)
   - ❌ TOML configuration file loading
   - ❌ Graceful shutdown with cleanup
   - ❌ Circuit breaker for repeated failures
   - ❌ Execution metrics and monitoring

4. **Optional Enhancements**
   - ❌ HTTP REST API for control (axum-based)
   - ❌ EventServer trait for event-driven execution
   - ❌ Parallel task execution with bounded pools
   - ❌ Connection pooling integration

---

## Phase 3: Persistent State Management

### Objective
Enable actor servers to survive restarts without losing task state or execution history.

### 3.1: Database Schema

**Location**: Create migration `migrations/YYYY-MM-DD-HHMMSS_create_actor_server_tables/`

```sql
-- migrations/.../up.sql
CREATE TABLE actor_server_state (
    task_id VARCHAR(255) PRIMARY KEY,
    actor_name VARCHAR(255) NOT NULL,
    last_run TIMESTAMPTZ,
    next_run TIMESTAMPTZ NOT NULL,
    consecutive_failures INTEGER DEFAULT 0,
    is_paused BOOLEAN DEFAULT FALSE,
    metadata JSONB DEFAULT '{}',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE actor_server_executions (
    id BIGSERIAL PRIMARY KEY,
    task_id VARCHAR(255) NOT NULL,
    actor_name VARCHAR(255) NOT NULL,
    started_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ,
    success BOOLEAN DEFAULT FALSE,
    error_message TEXT,
    skills_succeeded INTEGER DEFAULT 0,
    skills_failed INTEGER DEFAULT 0,
    skills_skipped INTEGER DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_actor_server_executions_task ON actor_server_executions(task_id);
CREATE INDEX idx_actor_server_executions_started ON actor_server_executions(started_at DESC);
CREATE INDEX idx_actor_server_state_next_run ON actor_server_state(next_run) WHERE NOT is_paused;
CREATE INDEX idx_actor_server_state_actor ON actor_server_state(actor_name);
```

### 3.2: Diesel Models

**Location**: `crates/botticelli_database/src/actor_server_models.rs`

```rust
use chrono::NaiveDateTime;
use diesel::prelude::*;
use derive_builder::Builder;
use derive_getters::Getters;

#[derive(Debug, Clone, Queryable, Identifiable, Selectable)]
#[diesel(table_name = crate::schema::actor_server_state)]
#[diesel(primary_key(task_id))]
pub struct ActorServerStateRow {
    pub task_id: String,
    pub actor_name: String,
    pub last_run: Option<NaiveDateTime>,
    pub next_run: NaiveDateTime,
    pub consecutive_failures: i32,
    pub is_paused: bool,
    pub metadata: serde_json::Value,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Insertable, Getters, Builder)]
#[diesel(table_name = crate::schema::actor_server_state)]
#[builder(setter(into))]
pub struct NewActorServerState {
    pub task_id: String,
    pub actor_name: String,
    #[builder(default)]
    pub last_run: Option<NaiveDateTime>,
    pub next_run: NaiveDateTime,
    #[builder(default)]
    pub consecutive_failures: i32,
    #[builder(default)]
    pub is_paused: bool,
    #[builder(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Queryable, Identifiable, Selectable)]
#[diesel(table_name = crate::schema::actor_server_executions)]
pub struct ActorServerExecutionRow {
    pub id: i64,
    pub task_id: String,
    pub actor_name: String,
    pub started_at: NaiveDateTime,
    pub completed_at: Option<NaiveDateTime>,
    pub success: bool,
    pub error_message: Option<String>,
    pub skills_succeeded: i32,
    pub skills_failed: i32,
    pub skills_skipped: i32,
    pub metadata: serde_json::Value,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Insertable, Getters, Builder)]
#[diesel(table_name = crate::schema::actor_server_executions)]
#[builder(setter(into))]
pub struct NewActorServerExecution {
    pub task_id: String,
    pub actor_name: String,
    pub started_at: NaiveDateTime,
    #[builder(default)]
    pub completed_at: Option<NaiveDateTime>,
    #[builder(default)]
    pub success: bool,
    #[builder(default)]
    pub error_message: Option<String>,
    #[builder(default)]
    pub skills_succeeded: i32,
    #[builder(default)]
    pub skills_failed: i32,
    #[builder(default)]
    pub skills_skipped: i32,
    #[builder(default)]
    pub metadata: serde_json::Value,
}
```

### 3.3: Database-Backed StatePersistence

**Location**: `crates/botticelli_actor/src/state_persistence.rs`

```rust
use botticelli_database::{establish_connection, ActorServerStateRow, NewActorServerState};
use botticelli_server::{ActorServerResult, StatePersistence};
use diesel::prelude::*;
use async_trait::async_trait;

pub struct DatabaseStatePersistence {
    database_url: String,
}

impl DatabaseStatePersistence {
    pub fn new(database_url: String) -> Self {
        Self { database_url }
    }
}

#[async_trait]
impl StatePersistence for DatabaseStatePersistence {
    type State = ActorServerStateRow;

    async fn save_state(&self, state: &Self::State) -> ActorServerResult<()> {
        let mut conn = establish_connection(&self.database_url)?;

        diesel::insert_into(actor_server_state::table)
            .values(state)
            .on_conflict(actor_server_state::task_id)
            .do_update()
            .set((
                actor_server_state::last_run.eq(&state.last_run),
                actor_server_state::next_run.eq(&state.next_run),
                actor_server_state::consecutive_failures.eq(&state.consecutive_failures),
                actor_server_state::is_paused.eq(&state.is_paused),
                actor_server_state::metadata.eq(&state.metadata),
                actor_server_state::updated_at.eq(diesel::dsl::now),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    async fn load_state(&self, task_id: &str) -> ActorServerResult<Option<Self::State>> {
        let mut conn = establish_connection(&self.database_url)?;

        let state = actor_server_state::table
            .find(task_id)
            .first::<ActorServerStateRow>(&mut conn)
            .optional()?;

        Ok(state)
    }
}
```

**Benefits**:
- Server restarts don't lose state
- Task history persists for auditing
- Can resume from last execution point
- Circuit breaker state survives crashes

**Implementation Steps**:
1. Create migration files
2. Run `diesel migration run`
3. Update schema.rs with new tables
4. Create Diesel models with builders
5. Implement DatabaseStatePersistence
6. Add state recovery to DiscordActorServer::start()
7. Add state persistence to task execution loop

---

## Phase 4: Advanced Scheduling

### Objective
Support cron expressions, sophisticated timing, and task lifecycle management.

### 4.1: Schedule Types

**Location**: `crates/botticelli_server/src/schedule.rs`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ScheduleType {
    /// Cron expression (e.g., "0 9 * * *" = 9 AM daily)
    Cron { expression: String },

    /// Fixed interval in seconds
    Interval { seconds: u64 },

    /// One-time execution at specific time
    Once { at: DateTime<Utc> },

    /// Execute immediately on startup
    Immediate,
}

pub struct ScheduleCheck {
    pub should_run: bool,
    pub next_run: DateTime<Utc>,
}

pub trait Schedule {
    fn check(&self, last_run: Option<DateTime<Utc>>) -> ScheduleCheck;
    fn next_execution(&self, after: DateTime<Utc>) -> DateTime<Utc>;
}
```

### 4.2: ScheduledServer Trait

**Location**: `crates/botticelli_server/src/scheduled.rs`

```rust
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct TaskStatus {
    pub id: String,
    pub name: String,
    pub is_paused: bool,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: DateTime<Utc>,
    pub consecutive_failures: u32,
}

#[async_trait]
pub trait ScheduledServer: Send + Sync {
    async fn execute_task(&mut self, task_id: &str) -> ActorServerResult<ExecutionResult>;
    async fn task_status(&self, task_id: &str) -> ActorServerResult<TaskStatus>;
    async fn list_tasks(&self) -> ActorServerResult<Vec<TaskStatus>>;
    async fn pause_task(&mut self, task_id: &str) -> ActorServerResult<()>;
    async fn resume_task(&mut self, task_id: &str) -> ActorServerResult<()>;
    async fn run_forever(&mut self) -> ActorServerResult<()>;
    async fn shutdown(&mut self) -> ActorServerResult<()>;
}
```

### 4.3: Cron Implementation

**Dependencies**: Add `cron = "0.12"` to Cargo.toml

**Location**: `crates/botticelli_actor/src/schedule_impl.rs`

Implement `Schedule` trait for `ScheduleType` using `cron` crate for expression parsing.

**Benefits**:
- Human-readable scheduling ("every day at 9am")
- Complex recurrence patterns
- Consistent interface for all schedule types
- Pause/resume for operational control

---

## Phase 5: Production Binary

### Objective
Deployable server executable with configuration and lifecycle management.

### 5.1: Binary Structure

**Location**: `crates/botticelli_actor/src/bin/actor-server.rs`

```rust
use botticelli_actor::{DiscordActorServer, ActorConfig};
use clap::Parser;
use tracing::info;

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value = "actor_server.toml")]
    config: String,

    #[arg(long, env = "DATABASE_URL")]
    database_url: String,

    #[arg(long, env = "DISCORD_TOKEN")]
    discord_token: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let args = Args::parse();
    info!("Starting Botticelli Actor Server");

    // Load configuration
    let config = load_server_config(&args.config)?;

    // Initialize server
    let mut server = DiscordActorServer::new(args.discord_token);

    // Load actors from config
    for actor_config in config.actors {
        let actor = load_actor(&actor_config)?;
        server.register_actor(actor).await?;
    }

    // Setup signal handling
    let shutdown = tokio::signal::ctrl_c();

    // Start server
    server.start().await?;

    // Wait for shutdown signal
    shutdown.await?;
    info!("Shutdown signal received");

    // Graceful shutdown
    server.stop().await?;

    Ok(())
}
```

### 5.2: Configuration Format

**File**: `examples/actor_server.toml`

```toml
[server]
check_interval_seconds = 60
max_consecutive_failures = 5

[[actors]]
name = "daily_poster"
config_file = "actors/daily_poster.toml"
channel_id = "1234567890"

[actors.schedule]
type = "Cron"
expression = "0 9 * * *"  # 9 AM daily

[[actors]]
name = "hourly_trends"
config_file = "actors/trending.toml"
channel_id = "0987654321"

[actors.schedule]
type = "Interval"
seconds = 3600  # Every hour
```

**Benefits**:
- Declarative actor configuration
- No code changes for new actors
- Easy operational control
- Environment variable support

---

## Phase 6: Observability & Operations

### 6.1: Metrics

Integrate with `prometheus` or `metrics` crate:

```rust
use metrics::{counter, histogram, gauge};

// In execute_task:
counter!("actor.executions.total", 1, "actor" => actor_name);
histogram!("actor.execution.duration_ms", duration.as_millis() as f64);

// In task scheduler:
gauge!("actor.tasks.scheduled", scheduled_count as f64);
gauge!("actor.tasks.paused", paused_count as f64);
```

### 6.2: Health Checks

**Location**: `crates/botticelli_actor/src/health.rs`

```rust
pub struct HealthStatus {
    pub healthy: bool,
    pub scheduled_tasks: usize,
    pub failed_tasks: Vec<String>,
    pub last_execution: Option<DateTime<Utc>>,
}

impl DiscordActorServer {
    pub async fn health(&self) -> HealthStatus {
        // Check all tasks, report status
    }
}
```

### 6.3: Alerting

Integration points for external alerting:
- Webhook on N consecutive failures
- PagerDuty integration for critical errors
- Discord notification to ops channel

---

## Phase 7: HTTP API (Optional)

### Objective
Remote control and monitoring via REST API.

**Location**: `crates/botticelli_actor/src/api.rs`

```rust
use axum::{Router, routing::get};

pub fn create_router(server: Arc<RwLock<DiscordActorServer>>) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/tasks", get(list_tasks))
        .route("/tasks/:id", get(get_task_status))
        .route("/tasks/:id/pause", post(pause_task))
        .route("/tasks/:id/resume", post(resume_task))
        .route("/tasks/:id/trigger", post(trigger_task))
        .route("/tasks/:id/history", get(task_history))
        .with_state(server)
}
```

**Endpoints**:
- `GET /health` - Server health status
- `GET /tasks` - List all scheduled tasks
- `GET /tasks/:id` - Task status details
- `POST /tasks/:id/trigger` - Manual execution
- `POST /tasks/:id/pause` - Pause scheduling
- `POST /tasks/:id/resume` - Resume scheduling
- `GET /tasks/:id/history` - Execution history

---

## Implementation Priority

### High Priority (Must Have for Production)

1. **Phase 3: Persistent State** (2-3 days)
   - Database schema and migrations
   - Diesel models with builders
   - DatabaseStatePersistence implementation
   - State recovery on startup

2. **Phase 5: Production Binary** (1-2 days)
   - Binary entry point with clap
   - TOML configuration loading
   - Graceful shutdown handling
   - Example configurations

### Medium Priority (Should Have)

3. **Phase 4: Advanced Scheduling** (2-3 days)
   - Schedule types and traits
   - Cron expression support
   - ScheduledServer trait
   - Pause/resume functionality

4. **Phase 6: Observability** (1-2 days)
   - Metrics instrumentation
   - Health check endpoints
   - Structured logging improvements

### Low Priority (Nice to Have)

5. **Phase 7: HTTP API** (2-3 days)
   - Axum router setup
   - REST endpoint implementations
   - API authentication
   - Documentation

6. **EventServer Trait** (Future)
   - Reactive event handling
   - Webhook integration
   - Real-time triggers

---

## Risk Assessment

### Technical Risks

1. **Database Connection Management**
   - **Risk**: PgConnection not Sync breaks async patterns
   - **Mitigation**: Use r2d2 or deadpool connection pooling

2. **Cron Parsing Edge Cases**
   - **Risk**: Invalid expressions or timezone issues
   - **Mitigation**: Validate on config load, default to UTC

3. **Task State Race Conditions**
   - **Risk**: Concurrent task execution with shared state
   - **Mitigation**: Use database locks or single-threaded scheduler

### Operational Risks

1. **Server Restart During Execution**
   - **Risk**: Tasks interrupted mid-execution
   - **Mitigation**: Idempotent operations, execution tracking

2. **Configuration Drift**
   - **Risk**: TOML file changes vs database state mismatch
   - **Mitigation**: Config validation, reload command

3. **Resource Exhaustion**
   - **Risk**: Too many scheduled tasks
   - **Mitigation**: Bounded task pools, resource limits

---

## Success Criteria

### Phase 3 Complete When:
- ✅ Server survives restart without losing task state
- ✅ Execution history viewable in database
- ✅ Circuit breaker prevents runaway failures

### Phase 5 Complete When:
- ✅ `actor-server` binary runs from TOML config
- ✅ Actors execute on configured schedules
- ✅ SIGTERM triggers graceful shutdown
- ✅ Example configs work out of box

### Production Ready When:
- ✅ State persistence working (Phase 3)
- ✅ Binary deployable (Phase 5)
- ✅ Metrics exposed (Phase 6)
- ✅ Integration tests passing
- ✅ Documentation complete

---

## Next Immediate Actions

1. **Create this strategy document** ✅ (current task)
2. **Review and approve** with user
3. **Start Phase 3**: Database schema migration
4. **Implement**: DatabaseStatePersistence
5. **Test**: State recovery after restart
6. **Iterate**: Based on results

## Questions for Consideration

1. **Connection Pooling**: Use `r2d2` or `deadpool`?
2. **Timezone Handling**: Store schedules in UTC or allow local time?
3. **Failure Strategy**: Circuit breaker only or also exponential backoff?
4. **API Priority**: Should HTTP API come before advanced scheduling?
5. **Testing Strategy**: How to integration test scheduled tasks?

---

**Last Updated**: 2025-11-23
**Next Review**: After Phase 3 completion
