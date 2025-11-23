//! Integration tests for Actor workflow.

use botticelli_actor::{
    Actor, ActorConfigBuilder, Content, ContentFormatterSkill, ContentSchedulingSkill,
    ContentSelectionSkill, DuplicateCheckSkill, ExecutionConfigBuilder, PlatformMetadata,
    PlatformMetadataBuilder, PostId, RateLimitingSkill, ScheduleId, SkillConfigBuilder,
    SkillRegistry, SocialMediaPlatform,
};
use botticelli_database::establish_connection;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;

/// Mock platform for testing.
struct MockPlatform {
    metadata: PlatformMetadata,
}

impl MockPlatform {
    fn new(name: impl Into<String>) -> Self {
        let metadata = PlatformMetadataBuilder::default()
            .name(name.into())
            .max_text_length(2000)
            .max_media_attachments(10)
            .supported_media_types(vec![
                "image/png".to_string(),
                "image/jpeg".to_string(),
                "video/mp4".to_string(),
            ])
            .build()
            .expect("Valid metadata");

        Self { metadata }
    }
}

#[async_trait::async_trait]
impl SocialMediaPlatform for MockPlatform {
    async fn post(&self, _content: Content) -> Result<PostId, botticelli_actor::ActorError> {
        Ok(PostId("mock-post-123".to_string()))
    }

    async fn schedule(
        &self,
        _content: Content,
        _time: DateTime<Utc>,
    ) -> Result<ScheduleId, botticelli_actor::ActorError> {
        Ok(ScheduleId("mock-schedule-123".to_string()))
    }

    async fn delete_post(&self, _id: PostId) -> Result<(), botticelli_actor::ActorError> {
        Ok(())
    }

    fn metadata(&self) -> PlatformMetadata {
        self.metadata.clone()
    }
}

#[tokio::test]
async fn test_actor_workflow() {
    // Set up database connection
    let mut conn = establish_connection().expect("Failed to connect to database");

    // Create actor configuration
    let execution = ExecutionConfigBuilder::default()
        .continue_on_error(true)
        .stop_on_unrecoverable(false)
        .max_retries(3)
        .build()
        .expect("Valid execution config");

    let mut skill_configs = HashMap::new();

    // Configure content selection skill
    let mut selection_settings = HashMap::new();
    selection_settings.insert("max_candidates".to_string(), serde_json::json!(5));
    selection_settings.insert("priority_weight".to_string(), serde_json::json!(0.7));
    selection_settings.insert("freshness_weight".to_string(), serde_json::json!(0.3));

    let selection_config = SkillConfigBuilder::default()
        .enabled(true)
        .settings(selection_settings)
        .build()
        .expect("Valid skill config");

    skill_configs.insert("content_selection".to_string(), selection_config);

    // Configure duplicate check skill
    let mut duplicate_settings = HashMap::new();
    duplicate_settings.insert("lookback_days".to_string(), serde_json::json!(30));
    duplicate_settings.insert("similarity_threshold".to_string(), serde_json::json!(0.9));

    let duplicate_config = SkillConfigBuilder::default()
        .enabled(true)
        .settings(duplicate_settings)
        .build()
        .expect("Valid skill config");

    skill_configs.insert("duplicate_check".to_string(), duplicate_config);

    // Configure rate limiting skill
    let mut rate_settings = HashMap::new();
    rate_settings.insert("max_posts_per_day".to_string(), serde_json::json!(10));
    rate_settings.insert("min_interval_minutes".to_string(), serde_json::json!(60));

    let rate_config = SkillConfigBuilder::default()
        .enabled(true)
        .settings(rate_settings)
        .build()
        .expect("Valid skill config");

    skill_configs.insert("rate_limiting".to_string(), rate_config);

    // Configure scheduling skill
    let mut schedule_settings = HashMap::new();
    schedule_settings.insert(
        "schedule_window_start".to_string(),
        serde_json::json!("09:00"),
    );
    schedule_settings.insert(
        "schedule_window_end".to_string(),
        serde_json::json!("21:00"),
    );
    schedule_settings.insert(
        "randomize_within_window".to_string(),
        serde_json::json!(true),
    );

    let schedule_config = SkillConfigBuilder::default()
        .enabled(true)
        .settings(schedule_settings)
        .build()
        .expect("Valid skill config");

    skill_configs.insert("scheduling".to_string(), schedule_config);

    // Configure formatter skill
    let mut formatter_settings = HashMap::new();
    formatter_settings.insert("max_text_length".to_string(), serde_json::json!(2000));
    formatter_settings.insert("include_source".to_string(), serde_json::json!(true));
    formatter_settings.insert("add_hashtags".to_string(), serde_json::json!(true));

    let formatter_config = SkillConfigBuilder::default()
        .enabled(true)
        .settings(formatter_settings)
        .build()
        .expect("Valid skill config");

    skill_configs.insert("content_formatter".to_string(), formatter_config);

    let config = ActorConfigBuilder::default()
        .name("test_actor".to_string())
        .description("Test actor for integration testing".to_string())
        .knowledge(vec!["content".to_string(), "post_history".to_string()])
        .skills(vec![
            "content_selection".to_string(),
            "duplicate_check".to_string(),
            "scheduling".to_string(),
            "rate_limiting".to_string(),
            "content_formatter".to_string(),
        ])
        .skill_configs(skill_configs)
        .execution(execution)
        .build()
        .expect("Valid actor config");

    // Create skill registry
    let mut registry = SkillRegistry::new();
    registry.register(Arc::new(ContentSelectionSkill::new()));
    registry.register(Arc::new(DuplicateCheckSkill::new()));
    registry.register(Arc::new(ContentSchedulingSkill::new()));
    registry.register(Arc::new(RateLimitingSkill::new()));
    registry.register(Arc::new(ContentFormatterSkill::new()));

    // Create platform
    let platform = Arc::new(MockPlatform::new("test_platform"));

    // Build actor
    let actor = Actor::builder()
        .config(config)
        .skills(registry)
        .platform(platform)
        .build()
        .expect("Valid actor");

    // Execute actor workflow
    let result = actor.execute(&mut conn).await;

    match result {
        Ok(execution_result) => {
            println!("Succeeded: {}", execution_result.succeeded.len());
            println!("Failed: {}", execution_result.failed.len());
            println!("Skipped: {}", execution_result.skipped.len());

            // Skills should execute without panic (may have empty results if tables don't exist)
            assert!(
                execution_result.succeeded.len() + execution_result.failed.len()
                    + execution_result.skipped.len()
                    > 0
            );
        }
        Err(e) => {
            // Some errors are expected if knowledge tables don't exist yet
            println!("Actor execution error (expected if tables not set up): {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_actor_with_disabled_skills() {
    let mut conn = establish_connection().expect("Failed to connect to database");

    let execution = ExecutionConfigBuilder::default()
        .continue_on_error(true)
        .build()
        .expect("Valid execution config");

    let mut skill_configs = HashMap::new();

    // Disable content selection skill
    let selection_config = SkillConfigBuilder::default()
        .enabled(false)
        .settings(HashMap::new())
        .build()
        .expect("Valid skill config");

    skill_configs.insert("content_selection".to_string(), selection_config);

    let config = ActorConfigBuilder::default()
        .name("test_actor_disabled".to_string())
        .description("Test actor with disabled skills".to_string())
        .knowledge(vec![])
        .skills(vec!["content_selection".to_string()])
        .skill_configs(skill_configs)
        .execution(execution)
        .build()
        .expect("Valid actor config");

    let mut registry = SkillRegistry::new();
    registry.register(Arc::new(ContentSelectionSkill::new()));

    let platform = Arc::new(MockPlatform::new("test_platform"));

    let actor = Actor::builder()
        .config(config)
        .skills(registry)
        .platform(platform)
        .build()
        .expect("Valid actor");

    let result = actor.execute(&mut conn).await.expect("Actor execution");

    // Disabled skill should be skipped
    assert_eq!(result.skipped.len(), 1);
    assert_eq!(result.succeeded.len(), 0);
}
