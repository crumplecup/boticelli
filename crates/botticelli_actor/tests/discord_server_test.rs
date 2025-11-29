use botticelli_actor::{
    ContentBuilder, ContentPostBuilder, DiscordActorId, DiscordActorManager, DiscordActorServer,
    DiscordContentPoster, DiscordContext, DiscordServerState, DiscordTaskScheduler,
};
use botticelli_server::{ActorServer, TaskScheduler};
use serenity::all::{ChannelId, Http};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_discord_actor_id_creation() {
    let channel_id = ChannelId::new(123456789);
    let id = DiscordActorId::new("test_actor", channel_id);

    assert_eq!(id.actor_name(), "test_actor");
    assert_eq!(id.channel_id(), channel_id);
}

#[tokio::test]
async fn test_discord_context_creation() {
    let http = Arc::new(Http::new("test_token"));
    let context = DiscordContext::new(http.clone());
    assert_eq!(Arc::strong_count(context.http()), 2);
}

#[tokio::test]
async fn test_task_scheduler_lifecycle() {
    let mut scheduler = DiscordTaskScheduler::new();

    // Task ID must be in format "actor_name:channel_id"
    let task_id = "test_actor:123456789".to_string();
    let duration = Duration::from_millis(100);

    let result = scheduler
        .schedule(task_id.clone(), duration, || async { Ok(()) })
        .await;
    assert!(result.is_ok());

    // Cancel the task
    let cancel_result = scheduler.cancel(&task_id).await;
    assert!(cancel_result.is_ok());
}

#[tokio::test]
async fn test_actor_manager_creation() {
    let _manager = DiscordActorManager::new();
    // Manager created successfully
}

#[tokio::test]
async fn test_content_poster_creation() {
    let http = Arc::new(Http::new("test_token"));
    let _poster = DiscordContentPoster::new(http);
}

#[tokio::test]
async fn test_server_state_persistence() {
    let mut last_executions = HashMap::new();
    last_executions.insert("actor1".to_string(), chrono::Utc::now());

    let mut execution_counts = HashMap::new();
    execution_counts.insert("actor1".to_string(), 5);

    let state = DiscordServerState {
        last_executions,
        execution_counts,
    };

    // Test JSON serialization
    let json = serde_json::to_string(&state).expect("Serialize state");
    assert!(json.contains("actor1"));

    // Test deserialization
    let deserialized: DiscordServerState = serde_json::from_str(&json).expect("Deserialize state");
    assert_eq!(deserialized.last_executions.len(), 1);
    assert_eq!(deserialized.execution_counts.len(), 1);
}

#[tokio::test]
async fn test_content_post_creation() {
    let content = ContentBuilder::default()
        .text(Some("Test content".to_string()))
        .build()
        .expect("Valid content");

    let post = ContentPostBuilder::default()
        .post_id("post456".to_string())
        .content(content)
        .destination("channel789".to_string())
        .build()
        .expect("Valid content post");

    assert_eq!(post.post_id(), "post456");
    assert_eq!(post.destination(), "channel789");
}

#[tokio::test]
async fn test_actor_server_creation() {
    let http = Arc::new(Http::new("test_token"));
    let temp_path = std::env::temp_dir().join("test_actor_server_state.json");
    let _server = DiscordActorServer::new(http, &temp_path);

    // Server created successfully
    let _ = std::fs::remove_file(temp_path);
}

#[tokio::test]
async fn test_actor_server_reload() {
    let http = Arc::new(Http::new("test_token"));
    let temp_path = std::env::temp_dir().join("test_reload_state.json");

    let mut last_executions = HashMap::new();
    last_executions.insert("actor1".to_string(), chrono::Utc::now());

    let state = DiscordServerState {
        last_executions,
        execution_counts: HashMap::new(),
    };

    // Write state file
    let json = serde_json::to_string(&state).expect("Serialize");
    std::fs::write(&temp_path, json).expect("Write state");

    let mut server = DiscordActorServer::new(http, &temp_path);

    // Reload should succeed
    let reload_result = server.reload().await;
    assert!(reload_result.is_ok());

    // Cleanup
    let _ = std::fs::remove_file(temp_path);
}
