//! Database-backed state persistence for actor servers.

use async_trait::async_trait;
use botticelli_database::{ActorServerStateRow, NewActorServerStateBuilder, establish_connection};
use botticelli_server::{ActorServerResult, StatePersistence};
use diesel::prelude::*;
use tracing::{debug, info, instrument};

/// Database-backed state persistence using PostgreSQL.
///
/// Stores actor server state in the `actor_server_state` table for
/// recovery after server restarts.
///
/// Note: Requires DATABASE_URL environment variable to be set.
#[derive(Debug, Clone)]
pub struct DatabaseStatePersistence;

impl DatabaseStatePersistence {
    /// Create a new database state persistence handler.
    ///
    /// Requires the DATABASE_URL environment variable to be set.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use botticelli_actor::DatabaseStatePersistence;
    ///
    /// // Requires DATABASE_URL=postgresql://localhost/botticelli in environment
    /// let persistence = DatabaseStatePersistence::new();
    /// ```
    pub fn new() -> Self {
        Self
    }
}

impl Default for DatabaseStatePersistence {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StatePersistence for DatabaseStatePersistence {
    type State = ActorServerStateRow;

    #[instrument(skip(self, state), fields(task_id = %state.task_id))]
    async fn save_state(&self, state: &Self::State) -> ActorServerResult<()> {
        debug!("Saving actor server state to database");

        // Run blocking database operation in dedicated thread pool
        let state = state.clone();

        tokio::task::spawn_blocking(move || -> ActorServerResult<()> {
            let mut conn = establish_connection().map_err(
                |e| -> Box<dyn std::error::Error + Send + Sync> {
                    format!("Failed to establish database connection: {}", e).into()
                },
            )?;

            // Use INSERT ... ON CONFLICT to upsert state
            diesel::insert_into(botticelli_database::schema::actor_server_state::table)
                .values(
                    &NewActorServerStateBuilder::default()
                        .task_id(&state.task_id)
                        .actor_name(&state.actor_name)
                        .last_run(state.last_run)
                        .next_run(state.next_run)
                        .consecutive_failures(state.consecutive_failures.unwrap_or(0))
                        .is_paused(state.is_paused.unwrap_or(false))
                        .metadata(state.metadata.clone().unwrap_or_default())
                        .build()
                        .expect("NewActorServerState with valid fields"),
                )
                .on_conflict(botticelli_database::schema::actor_server_state::task_id)
                .do_update()
                .set((
                    botticelli_database::schema::actor_server_state::last_run.eq(&state.last_run),
                    botticelli_database::schema::actor_server_state::next_run.eq(&state.next_run),
                    botticelli_database::schema::actor_server_state::consecutive_failures
                        .eq(&state.consecutive_failures),
                    botticelli_database::schema::actor_server_state::is_paused.eq(&state.is_paused),
                    botticelli_database::schema::actor_server_state::metadata.eq(&state.metadata),
                    botticelli_database::schema::actor_server_state::updated_at
                        .eq(diesel::dsl::now),
                ))
                .execute(&mut conn)
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                    format!("Failed to save state: {}", e).into()
                })?;

            info!("Actor server state saved to database");
            Ok(())
        })
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
            format!("Task join error: {}", e).into()
        })?
    }

    #[instrument(skip(self))]
    async fn load_state(&self) -> ActorServerResult<Option<Self::State>> {
        debug!("Loading actor server state from database");

        tokio::task::spawn_blocking(move || -> ActorServerResult<Option<ActorServerStateRow>> {
            let mut conn = establish_connection().map_err(
                |e| -> Box<dyn std::error::Error + Send + Sync> {
                    format!("Failed to establish database connection: {}", e).into()
                },
            )?;

            // Load all state rows (for now, just get the first one)
            let states = botticelli_database::schema::actor_server_state::table
                .load::<ActorServerStateRow>(&mut conn)
                .optional()
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                    format!("Failed to load state: {}", e).into()
                })?;

            if let Some(states_vec) = states {
                if !states_vec.is_empty() {
                    info!(count = states_vec.len(), "Loaded actor server states");
                    Ok(Some(states_vec.into_iter().next().unwrap()))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        })
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
            format!("Task join error: {}", e).into()
        })?
    }

    #[instrument(skip(self))]
    async fn clear_state(&self) -> ActorServerResult<()> {
        debug!("Clearing all actor server state from database");

        tokio::task::spawn_blocking(move || -> ActorServerResult<()> {
            let mut conn = establish_connection().map_err(
                |e| -> Box<dyn std::error::Error + Send + Sync> {
                    format!("Failed to establish database connection: {}", e).into()
                },
            )?;

            diesel::delete(botticelli_database::schema::actor_server_state::table)
                .execute(&mut conn)
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                    format!("Failed to clear state: {}", e).into()
                })?;

            info!("Cleared all actor server state");
            Ok(())
        })
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
            format!("Task join error: {}", e).into()
        })?
    }
}
