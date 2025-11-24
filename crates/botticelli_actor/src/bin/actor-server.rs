//! Botticelli Actor Server - Long-running social media automation server.
//!
//! This binary runs actor servers that execute scheduled tasks for social media
//! platforms like Discord, posting content based on narratives and knowledge tables.

#[cfg(feature = "discord")]
use botticelli_actor::{Actor, ActorConfig, ScheduleConfig, SkillRegistry};
use botticelli_actor::{ActorServerConfig, DatabaseStatePersistence};
#[cfg(feature = "discord")]
use botticelli_database::establish_connection;
#[cfg(feature = "discord")]
use botticelli_server::ActorServer;
#[cfg(feature = "discord")]
use botticelli_server::Schedule;
use botticelli_server::StatePersistence;
use clap::Parser;
#[cfg(feature = "discord")]
use std::collections::HashMap;
use std::path::PathBuf;
#[cfg(feature = "discord")]
use std::sync::Arc;
use tracing::info;
use tracing::warn;
#[cfg(feature = "discord")]
use tracing::{debug, error};
use tracing_subscriber::EnvFilter;

#[cfg(feature = "discord")]
use botticelli_actor::{DiscordActorServer, DiscordPlatform};

#[cfg(feature = "discord")]
use serenity::http::Http;

#[cfg(feature = "discord")]
use chrono::{DateTime, Utc};

/// Command-line arguments for the actor server.
#[derive(Parser, Debug)]
#[command(name = "actor-server")]
#[command(about = "Botticelli Actor Server - Social media automation")]
#[command(version)]
struct Args {
    /// Path to server configuration file
    #[arg(short, long, default_value = "actor_server.toml")]
    config: PathBuf,

    /// Database URL for state persistence
    #[arg(long, env = "DATABASE_URL")]
    database_url: Option<String>,

    /// Discord bot token
    #[arg(long, env = "DISCORD_TOKEN")]
    #[cfg(feature = "discord")]
    discord_token: Option<String>,

    /// Dry run mode (don't actually execute actors)
    #[arg(long)]
    dry_run: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();
    info!("Starting Botticelli Actor Server");
    info!(config_file = ?args.config, "Loading configuration");

    // Load server configuration
    let server_config = ActorServerConfig::from_file(&args.config)?;
    info!(
        actors = server_config.actors.len(),
        check_interval = server_config.server.check_interval_seconds,
        "Configuration loaded"
    );

    if args.dry_run {
        info!("DRY RUN MODE - No actions will be executed");
        // Just validate configuration and exit
        for actor_instance in &server_config.actors {
            info!(
                actor = %actor_instance.name,
                config = %actor_instance.config_file,
                enabled = actor_instance.enabled,
                "Actor configuration validated"
            );
        }
        info!("Configuration validation complete");
        return Ok(());
    }

    // Set up database state persistence if DATABASE_URL is set
    if args.database_url.is_some() || std::env::var("DATABASE_URL").is_ok() {
        info!("Database state persistence enabled");
        let persistence = DatabaseStatePersistence::new()
            .map_err(|e| format!("Failed to create persistence: {}", e))?;

        // Attempt to load previous state
        match persistence.load_state().await {
            Ok(Some(state)) => {
                info!(
                    task_id = %state.task_id,
                    actor = %state.actor_name,
                    "Loaded previous server state from database"
                );
            }
            Ok(None) => {
                info!("No previous server state found in database");
            }
            Err(e) => {
                warn!("Failed to load previous state: {}", e);
            }
        }
    } else {
        warn!("DATABASE_URL not set - state persistence disabled");
    }

    #[cfg(feature = "discord")]
    {
        // Initialize Discord server
        let discord_token = args
            .discord_token
            .or_else(|| std::env::var("DISCORD_TOKEN").ok())
            .ok_or("DISCORD_TOKEN not provided")?;

        // Create Discord HTTP client
        let http = Arc::new(Http::new(&discord_token));

        // Initialize server with state file path
        let state_path = PathBuf::from(".actor_server_state.json");
        let mut server = DiscordActorServer::new(http.clone(), state_path);

        // Track actors and their schedules
        let mut actors: HashMap<String, (Actor, ScheduleConfig, Option<DateTime<Utc>>)> =
            HashMap::new();

        // Load and register actors from configuration
        for actor_instance in &server_config.actors {
            if !actor_instance.enabled {
                info!(actor = %actor_instance.name, "Actor disabled, skipping");
                continue;
            }

            info!(
                actor = %actor_instance.name,
                config_file = %actor_instance.config_file,
                "Loading actor"
            );

            // Load actor configuration
            let actor_config = ActorConfig::from_file(&actor_instance.config_file)?;

            // Create Discord platform if channel_id is provided
            if let Some(channel_id) = &actor_instance.channel_id {
                let platform = DiscordPlatform::new(&discord_token, channel_id)?;

                // Build actor with platform
                let actor = Actor::builder()
                    .config(actor_config)
                    .skills(SkillRegistry::new())
                    .platform(std::sync::Arc::new(platform))
                    .build()?;

                info!(actor = %actor_instance.name, "Actor created successfully");

                // Store actor with schedule
                actors.insert(
                    actor_instance.name.clone(),
                    (actor, actor_instance.schedule.clone(), None),
                );

                match &actor_instance.schedule {
                    ScheduleConfig::Interval { seconds } => {
                        info!(
                            actor = %actor_instance.name,
                            interval_seconds = seconds,
                            "Scheduled with interval"
                        );
                    }
                    ScheduleConfig::Immediate => {
                        info!(
                            actor = %actor_instance.name,
                            "Scheduled for immediate execution"
                        );
                    }
                    ScheduleConfig::Cron { expression } => {
                        info!(
                            actor = %actor_instance.name,
                            cron = expression,
                            "Scheduled with cron"
                        );
                    }
                    ScheduleConfig::Once { at } => {
                        info!(
                            actor = %actor_instance.name,
                            scheduled_at = at,
                            "Scheduled for one-time execution"
                        );
                    }
                }
            } else {
                warn!(
                    actor = %actor_instance.name,
                    "No channel_id specified, actor will not post"
                );
            }
        }

        // Set up graceful shutdown signal handler
        let shutdown_flag = Arc::new(tokio::sync::Notify::new());
        let shutdown_flag_clone = shutdown_flag.clone();

        tokio::spawn(async move {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to install CTRL+C signal handler");
            shutdown_flag_clone.notify_one();
        });

        info!("Actor server starting");

        // Start the server
        server
            .start()
            .await
            .map_err(|e| format!("Failed to start server: {}", e))?;

        info!("Actor server running. Press CTRL+C to shutdown.");

        // Main execution loop
        let check_interval =
            std::time::Duration::from_secs(server_config.server.check_interval_seconds);
        let mut interval = tokio::time::interval(check_interval);

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    debug!("Checking for ready actors");

                    // Check each actor's schedule
                    for (name, (actor, schedule, last_run)) in actors.iter_mut() {
                        let check = schedule.check(*last_run);

                        if check.should_run {
                            info!(actor = %name, "Executing scheduled actor");

                            // Get database connection
                            match establish_connection() {
                                Ok(mut conn) => {
                                    // Execute the actor
                                    match actor.execute(&mut conn).await {
                                        Ok(_) => {
                                            info!(actor = %name, "Actor executed successfully");
                                            *last_run = Some(Utc::now());
                                        }
                                        Err(e) => {
                                            error!(actor = %name, error = ?e, "Actor execution failed");
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!(actor = %name, error = ?e, "Failed to establish database connection");
                                }
                            }

                            if let Some(next) = check.next_run {
                                debug!(actor = %name, next_run = %next, "Next execution scheduled");
                            }
                        }
                    }
                }
                _ = shutdown_flag.notified() => {
                    info!("Shutdown signal received, stopping gracefully...");
                    break;
                }
            }
        }

        // Graceful shutdown
        server
            .stop()
            .await
            .map_err(|e| format!("Failed to stop server: {}", e))?;

        info!("Actor server stopped successfully");
        Ok(())
    }

    #[cfg(not(feature = "discord"))]
    {
        eprintln!("Discord feature not enabled. Rebuild with --features discord");
        Err("Discord feature required".into())
    }
}
