use botticelli_error::{BotticelliResult, ConfigError};
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use typed_builder::TypedBuilder;

/// Configuration for the bot server.
#[derive(Debug, Clone, Serialize, Deserialize, Getters, TypedBuilder)]
pub struct BotConfig {
    /// Generation bot configuration
    #[builder(setter(into))]
    generation: GenerationConfig,
    /// Curation bot configuration
    #[builder(setter(into))]
    curation: CurationConfig,
    /// Posting bot configuration
    #[builder(setter(into))]
    posting: PostingConfig,
}

impl BotConfig {
    /// Load bot configuration from a TOML file.
    #[tracing::instrument(skip(path))]
    pub fn from_file(path: impl AsRef<Path>) -> BotticelliResult<Self> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            botticelli_error::BotticelliError::from(ConfigError::new(format!(
                "Failed to read config file: {}",
                e
            )))
        })?;

        toml::from_str(&content).map_err(|e| {
            botticelli_error::BotticelliError::from(ConfigError::new(format!(
                "Failed to parse config: {}",
                e
            )))
        })
    }
}

/// Configuration for the generation bot.
#[derive(Debug, Clone, Serialize, Deserialize, Getters, TypedBuilder)]
pub struct GenerationConfig {
    /// Path to generation narrative TOML
    #[builder(setter(into))]
    narrative_path: PathBuf,
    /// Name of narrative within file
    #[builder(setter(into))]
    narrative_name: String,
    /// How often to run generation (hours)
    interval_hours: u64,
}

/// Configuration for the curation bot.
#[derive(Debug, Clone, Serialize, Deserialize, Getters, TypedBuilder)]
pub struct CurationConfig {
    /// Path to curation narrative TOML
    #[builder(setter(into))]
    narrative_path: PathBuf,
    /// Name of narrative within file
    #[builder(setter(into))]
    narrative_name: String,
    /// How often to check for new content (hours)
    #[serde(default)]
    #[builder(default, setter(strip_option))]
    check_interval_hours: Option<u64>,
    /// How often to check for new content (minutes, for testing)
    #[serde(default)]
    #[builder(default, setter(strip_option))]
    check_interval_minutes: Option<u64>,
    /// Batch size for processing
    batch_size: usize,
}

/// Configuration for the posting bot.
#[derive(Debug, Clone, Serialize, Deserialize, Getters, TypedBuilder)]
pub struct PostingConfig {
    /// Path to posting narrative TOML
    #[builder(setter(into))]
    narrative_path: PathBuf,
    /// Name of narrative within file
    #[builder(setter(into))]
    narrative_name: String,
    /// Base interval between posts (hours)
    base_interval_hours: u64,
    /// Maximum jitter to add (±minutes)
    jitter_minutes: u64,
}

/// Bot scheduling configuration.
#[derive(Debug, Clone, Getters, TypedBuilder)]
pub struct BotSchedule {
    /// Generation interval
    generation_interval: std::time::Duration,
    /// Curation check interval
    curation_interval: std::time::Duration,
    /// Posting base interval
    posting_base_interval: std::time::Duration,
    /// Posting jitter range
    posting_jitter: std::time::Duration,
}

impl From<&BotConfig> for BotSchedule {
    fn from(config: &BotConfig) -> Self {
        // Prefer minutes over hours for curation if specified
        let curation_secs = if let Some(mins) = config.curation().check_interval_minutes() {
            *mins * 60
        } else if let Some(hours) = config.curation().check_interval_hours() {
            *hours * 3600
        } else {
            12 * 3600 // Default to 12 hours
        };

        Self::builder()
            .generation_interval(std::time::Duration::from_secs(
                *config.generation().interval_hours() * 3600,
            ))
            .curation_interval(std::time::Duration::from_secs(curation_secs))
            .posting_base_interval(std::time::Duration::from_secs(
                *config.posting().base_interval_hours() * 3600,
            ))
            .posting_jitter(std::time::Duration::from_secs(
                *config.posting().jitter_minutes() * 60,
            ))
            .build()
    }
}
