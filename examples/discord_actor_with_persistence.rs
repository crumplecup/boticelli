//! Example showing how to integrate DatabaseStatePersistence with actor execution.
//!
//! This demonstrates the complete lifecycle of:
//! 1. Creating a persistence backend with connection pooling
//! 2. Using ActorExecutionTracker to manage execution state
//! 3. Handling circuit breaker logic
//! 4. Recording success/failure with execution metadata
//! 5. Recovering state after restart

use botticelli_actor::{
    ActorExecutionTracker, DatabaseExecutionResult, DatabaseStatePersistence,
};
use botticelli_server::ActorServerResult;
use chrono::Utc;
use std::sync::Arc;
use tracing::{error, info};

/// Simulated actor that posts content to Discord
struct DiscordActor {
    name: String,
}

impl DiscordActor {
    fn new(name: String) -> Self {
        Self { name }
    }

    /// Execute actor skills (simulated)
    async fn execute_skills(&self) -> Result<DatabaseExecutionResult, Box<dyn std::error::Error>> {
        // Simulate some work
        info!("Executing Discord posting skills for actor: {}", self.name);

        // In real code, this would:
        // 1. Query knowledge base for content
        // 2. Generate post using AI
        // 3. Post to Discord channel
        // 4. Track which skills succeeded/failed

        Ok(DatabaseExecutionResult {
            skills_succeeded: 3,
            skills_failed: 0,
            skills_skipped: 0,
            metadata: serde_json::json!({
                "channel_id": "123456789",
                "message_id": "987654321",
                "content_length": 280,
            }),
        })
    }

    /// Execute with persistence tracking
    async fn execute_with_persistence<P>(
        &self,
        tracker: &ActorExecutionTracker<P>,
    ) -> ActorServerResult<()>
    where
        P: botticelli_server::StatePersistence,
    {
        // This is the integration pattern for DatabaseStatePersistence
        // For now, we'll just document the pattern
        info!(
            "Would execute with persistence for task: {}",
            tracker.task_id()
        );
        Ok(())
    }
}

// This example demonstrates the pattern but requires database setup
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Discord Actor with Persistence Example");
    info!("This example demonstrates the integration pattern.");
    info!("To run with real database, set DATABASE_URL environment variable.");

    // Example pattern for using DatabaseStatePersistence
    demonstrate_usage_pattern().await;

    Ok(())
}

async fn demonstrate_usage_pattern() {
    info!("\n=== Usage Pattern ===\n");

    info!("1. Create persistence backend:");
    info!("   let persistence = Arc::new(DatabaseStatePersistence::new()?);");

    info!("\n2. Create execution tracker:");
    info!("   let tracker = ActorExecutionTracker::new(");
    info!("       persistence.clone(),");
    info!("       \"daily-discord-post\".to_string(),");
    info!("       \"discord-content-actor\".to_string(),");
    info!("   );");

    info!("\n3. Check circuit breaker before execution:");
    info!("   if !tracker.should_execute().await? {{");
    info!("       info!(\"Task paused or circuit broken\");");
    info!("       return Ok(());");
    info!("   }}");

    info!("\n4. Start execution tracking:");
    info!("   let exec_id = tracker.start_execution().await?;");

    info!("\n5. Execute actor skills:");
    info!("   match actor.execute_skills().await {{");
    info!("       Ok(result) => {{");
    info!("           tracker.record_success(exec_id, result).await?;");
    info!("           info!(\"Execution succeeded\");");
    info!("       }}");
    info!("       Err(e) => {{");
    info!("           let should_pause = tracker");
    info!("               .record_failure(exec_id, &e.to_string())");
    info!("               .await?;");
    info!("           if should_pause {{");
    info!("               error!(\"Circuit breaker triggered - task paused\");");
    info!("           }}");
    info!("           return Err(e.into());");
    info!("       }}");
    info!("   }}");

    info!("\n6. Update next run time:");
    info!("   let next_run = (Utc::now() + Duration::hours(24)).naive_utc();");
    info!("   tracker.update_next_run(next_run).await?;");

    info!("\n=== Recovery After Restart ===\n");

    info!("On server startup:");
    info!("   let persistence = DatabaseStatePersistence::new()?;");
    info!("   let active_tasks = persistence.list_active_tasks().await?;");
    info!("");
    info!("   for state in active_tasks {{");
    info!("       // Re-create actor from persisted state");
    info!("       let actor = DiscordActor::new(state.actor_name.clone());");
    info!("       ");
    info!("       // Re-schedule using persisted interval");
    info!("       let interval = Duration::seconds(state.interval_seconds);");
    info!("       scheduler.schedule(");
    info!("           state.task_id,");
    info!("           interval,");
    info!("           move || async {{ actor.execute().await }}");
    info!("       ).await?;");
    info!("   }}");

    info!("\n=== Circuit Breaker Behavior ===\n");

    info!("- Consecutive failures are tracked per task");
    info!("- When threshold exceeded (default 10), task auto-pauses");
    info!("- Success resets failure counter");
    info!("- Use CLI to manually pause/resume:");
    info!("  $ actor-server state pause <task-id>");
    info!("  $ actor-server state resume <task-id>");
    info!("  $ actor-server state list");
}
