//! Server configuration for actor-server binary.

use botticelli_server::{Schedule, ScheduleCheck};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Top-level server configuration loaded from TOML file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorServerConfig {
    /// Server-level settings
    #[serde(default)]
    pub server: ServerSettings,
    /// Actor configurations
    #[serde(default)]
    pub actors: Vec<ActorInstanceConfig>,
}

impl ActorServerConfig {
    /// Load server configuration from a TOML file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the TOML configuration file
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be read or TOML is invalid.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&contents)?;
        Ok(config)
    }
}

/// Server-level settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSettings {
    /// Interval in seconds between checking scheduled tasks
    #[serde(default = "default_check_interval")]
    pub check_interval_seconds: u64,
    /// Circuit breaker configuration
    #[serde(default)]
    pub circuit_breaker: CircuitBreakerConfig,
}

/// Circuit breaker configuration for automatic task pause on repeated failures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Maximum consecutive failures before action is taken
    #[serde(default = "default_max_failures")]
    pub max_consecutive_failures: i32,
    /// Whether to automatically pause tasks that exceed failure threshold
    #[serde(default = "default_auto_pause")]
    pub auto_pause: bool,
    /// Whether successful execution resets the failure counter
    #[serde(default = "default_reset_on_success")]
    pub reset_on_success: bool,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            max_consecutive_failures: default_max_failures(),
            auto_pause: default_auto_pause(),
            reset_on_success: default_reset_on_success(),
        }
    }
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            check_interval_seconds: default_check_interval(),
            circuit_breaker: CircuitBreakerConfig::default(),
        }
    }
}

fn default_check_interval() -> u64 {
    60
}

fn default_max_failures() -> i32 {
    5
}

fn default_auto_pause() -> bool {
    true
}

fn default_reset_on_success() -> bool {
    true
}

/// Configuration for a single actor instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorInstanceConfig {
    /// Actor name (unique identifier)
    pub name: String,
    /// Path to the actor's configuration TOML file
    pub config_file: String,
    /// Discord channel ID for posting
    #[serde(default)]
    pub channel_id: Option<String>,
    /// Task scheduling configuration
    #[serde(default)]
    pub schedule: ScheduleConfig,
    /// Whether this actor is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

/// Task scheduling configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ScheduleConfig {
    /// Fixed interval in seconds
    Interval {
        /// Interval duration in seconds
        seconds: u64,
    },
    /// Cron expression (for future Phase 4 implementation)
    #[allow(dead_code)]
    Cron {
        /// Cron expression string
        expression: String,
    },
    /// One-time execution at specific time
    #[allow(dead_code)]
    Once {
        /// ISO 8601 timestamp
        at: String,
    },
    /// Execute immediately on startup
    Immediate,
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        Self::Interval { seconds: 3600 }
    }
}

impl Schedule for ScheduleConfig {
    fn check(&self, last_run: Option<DateTime<Utc>>) -> ScheduleCheck {
        match self {
            ScheduleConfig::Interval { seconds } => {
                let now = Utc::now();
                let interval = Duration::seconds(*seconds as i64);

                match last_run {
                    None => {
                        // Never run before, run now and schedule next
                        let next = now + interval;
                        ScheduleCheck::run_and_schedule(next)
                    }
                    Some(last) => {
                        let next = last + interval;
                        if now >= next {
                            // Interval has elapsed, run now
                            ScheduleCheck::run_and_schedule(now + interval)
                        } else {
                            // Wait until next scheduled time
                            ScheduleCheck::wait_until(next)
                        }
                    }
                }
            }
            ScheduleConfig::Cron { expression } => {
                let now = Utc::now();

                // Parse cron expression
                let schedule = match expression.parse::<cron::Schedule>() {
                    Ok(s) => s,
                    Err(_) => {
                        // Invalid cron expression, never run
                        return ScheduleCheck::new(false, None);
                    }
                };

                // Find next execution time from now
                let next = schedule.after(&now).next();

                match (last_run, next) {
                    (None, Some(next_time)) => {
                        // Never run before, check if it's time
                        if now >= next_time {
                            // Time to run
                            let after_next = schedule.after(&next_time).next();
                            ScheduleCheck::new(true, after_next)
                        } else {
                            // Wait until scheduled time
                            ScheduleCheck::wait_until(next_time)
                        }
                    }
                    (Some(last), Some(_next_time)) => {
                        // Find next occurrence after last run
                        let next_after_last = schedule.after(&last).next();

                        match next_after_last {
                            Some(next_run) if now >= next_run => {
                                // Time to run
                                let after_next = schedule.after(&next_run).next();
                                ScheduleCheck::new(true, after_next)
                            }
                            Some(next_run) => {
                                // Wait until next scheduled time
                                ScheduleCheck::wait_until(next_run)
                            }
                            None => {
                                // No more scheduled runs
                                ScheduleCheck::new(false, None)
                            }
                        }
                    }
                    _ => {
                        // No next time available
                        ScheduleCheck::new(false, None)
                    }
                }
            }
            ScheduleConfig::Once { at } => {
                // Parse ISO 8601 timestamp
                let scheduled_time = match DateTime::parse_from_rfc3339(at) {
                    Ok(dt) => dt.with_timezone(&Utc),
                    Err(_) => {
                        // Invalid timestamp, never run
                        return ScheduleCheck::new(false, None);
                    }
                };

                match last_run {
                    None => {
                        let now = Utc::now();
                        if now >= scheduled_time {
                            // Time to run once
                            ScheduleCheck::run_once()
                        } else {
                            // Wait until scheduled time
                            ScheduleCheck::wait_until(scheduled_time)
                        }
                    }
                    Some(_) => {
                        // Already run, never run again
                        ScheduleCheck::new(false, None)
                    }
                }
            }
            ScheduleConfig::Immediate => {
                match last_run {
                    None => {
                        // Never run before, run immediately
                        ScheduleCheck::run_once()
                    }
                    Some(_) => {
                        // Already run, never run again
                        ScheduleCheck::new(false, None)
                    }
                }
            }
        }
    }

    fn next_execution(&self, after: DateTime<Utc>) -> Option<DateTime<Utc>> {
        match self {
            ScheduleConfig::Interval { seconds } => {
                let interval = Duration::seconds(*seconds as i64);
                Some(after + interval)
            }
            ScheduleConfig::Cron { expression } => {
                let schedule = expression.parse::<cron::Schedule>().ok()?;
                schedule.after(&after).next()
            }
            ScheduleConfig::Once { at } => {
                let scheduled_time = DateTime::parse_from_rfc3339(at).ok()?.with_timezone(&Utc);

                if after < scheduled_time {
                    Some(scheduled_time)
                } else {
                    None
                }
            }
            ScheduleConfig::Immediate => None,
        }
    }
}
