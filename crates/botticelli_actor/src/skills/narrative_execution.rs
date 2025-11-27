//! Narrative execution skill for running narrative workflows.

use crate::{ActorError, ActorErrorKind, Skill, SkillContext, SkillOutput, SkillResult};
use async_trait::async_trait;
use botticelli_narrative::{MultiNarrative, Narrative, NarrativeProvider};
use serde_json::json;
use std::path::Path;

/// Skill for executing narrative workflows.
pub struct NarrativeExecutionSkill {
    name: String,
}

impl NarrativeExecutionSkill {
    /// Create a new narrative execution skill.
    pub fn new() -> Self {
        Self {
            name: "narrative_execution".to_string(),
        }
    }
}

impl Default for NarrativeExecutionSkill {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for NarrativeExecutionSkill {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Execute narrative workflows using botticelli_narrative"
    }

    #[tracing::instrument(skip(self, context), fields(skill = %self.name))]
    async fn execute(&self, context: &SkillContext) -> SkillResult<SkillOutput> {
        tracing::debug!("Executing narrative execution skill");

        let narrative_path = context
            .config
            .get("narrative_path")
            .ok_or_else(|| {
                ActorError::new(ActorErrorKind::InvalidConfiguration(
                    "Missing narrative_path configuration".to_string(),
                ))
            })?;

        let narrative_name = context.config.get("narrative_name");

        tracing::info!(
            narrative_path,
            narrative_name = ?narrative_name,
            "Loading narrative for execution"
        );

        // Load narrative from file
        let path = Path::new(narrative_path);
        
        let narrative = if let Some(name) = narrative_name.as_ref() {
            // Load specific narrative from multi-narrative file
            tracing::debug!(narrative_name = name, "Loading specific narrative from file");
            let multi = MultiNarrative::from_file(path, name).map_err(|e| {
                ActorError::new(ActorErrorKind::FileIo {
                    path: path.to_path_buf(),
                    message: format!("Failed to load multi-narrative file: {}", e),
                })
            })?;
            
            multi.get_narrative(name).ok_or_else(|| {
                ActorError::new(ActorErrorKind::InvalidConfiguration(
                    format!("Narrative '{}' not found in file", name)
                ))
            })?.clone()
        } else {
            // Load single narrative file
            tracing::debug!("Loading single narrative from file");
            Narrative::from_file(path).map_err(|e| {
                ActorError::new(ActorErrorKind::FileIo {
                    path: path.to_path_buf(),
                    message: format!("Failed to load narrative: {}", e),
                })
            })?
        };

        tracing::debug!(
            narrative_name = narrative.name(),
            act_count = narrative.acts().len(),
            "Narrative loaded successfully"
        );

        // TODO: Get database connection from SkillContext
        // Currently SkillContext doesn't provide a way to access the database connection
        // This needs to be added to SkillContext or passed through config
        
        tracing::warn!("Database connection not available in SkillContext");
        tracing::warn!("Narrative execution requires database access - not yet implemented");

        Ok(SkillOutput {
            skill_name: self.name.clone(),
            data: json!({
                "status": "loaded_but_not_executed",
                "narrative_path": narrative_path,
                "narrative_name": narrative.name(),
                "act_count": narrative.acts().len(),
                "note": "Database connection not available in SkillContext",
            }),
        })
    }
}
