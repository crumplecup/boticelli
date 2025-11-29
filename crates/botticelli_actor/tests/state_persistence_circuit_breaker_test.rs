//! Tests for circuit breaker functionality in DatabaseStatePersistence.

use botticelli_actor::{DatabaseExecutionResult, DatabaseStatePersistence};
use botticelli_database::{ActorServerStateRow, establish_connection};
use chrono::Utc;
use diesel::prelude::*;

fn setup_test_task(task_id: &str, actor_name: &str) -> ActorServerStateRow {
    ActorServerStateRow {
        task_id: task_id.to_string(),
        actor_name: actor_name.to_string(),
        last_run: None,
        next_run: Utc::now().naive_utc(),
        consecutive_failures: Some(0),
        is_paused: Some(false),
        metadata: Some(serde_json::json!({})),
        updated_at: Utc::now().naive_utc(),
    }
}

fn cleanup_task(task_id: &str) {
    let mut conn = establish_connection().expect("Database connection");

    // Clean up execution log entries
    diesel::delete(
        botticelli_database::schema::actor_server_executions::table
            .filter(botticelli_database::schema::actor_server_executions::task_id.eq(task_id)),
    )
    .execute(&mut conn)
    .ok();

    // Clean up state entries
    diesel::delete(
        botticelli_database::schema::actor_server_state::table
            .filter(botticelli_database::schema::actor_server_state::task_id.eq(task_id)),
    )
    .execute(&mut conn)
    .ok();
}

#[tokio::test]
async fn test_record_failure_increments_counter() {
    dotenvy::dotenv().ok();
    let persistence =
        DatabaseStatePersistence::with_pool_size(2).expect("Failed to create persistence");
    let task_id = "test_failure_counter";
    let state = setup_test_task(task_id, "test_actor");

    persistence
        .save_task_state(task_id, &state)
        .await
        .expect("Save initial state");

    // Record first failure
    let threshold_exceeded = persistence
        .record_failure(task_id, 3)
        .await
        .expect("Record failure");
    assert!(!threshold_exceeded, "Should not exceed threshold yet");

    // Check counter
    let loaded = persistence
        .load_task_state(task_id)
        .await
        .expect("Load state")
        .expect("State exists");
    assert_eq!(loaded.consecutive_failures, Some(1));

    // Record second failure
    persistence
        .record_failure(task_id, 3)
        .await
        .expect("Record failure");

    let loaded = persistence
        .load_task_state(task_id)
        .await
        .expect("Load state")
        .expect("State exists");
    assert_eq!(loaded.consecutive_failures, Some(2));

    cleanup_task(task_id);
}

#[tokio::test]
async fn test_record_failure_exceeds_threshold() {
    dotenvy::dotenv().ok();
    let persistence =
        DatabaseStatePersistence::with_pool_size(2).expect("Failed to create persistence");
    let task_id = "test_threshold";
    let state = setup_test_task(task_id, "test_actor");

    persistence
        .save_task_state(task_id, &state)
        .await
        .expect("Save initial state");

    // Record failures up to threshold
    for i in 1..=2 {
        let exceeded = persistence
            .record_failure(task_id, 3)
            .await
            .expect("Record failure");
        assert!(!exceeded, "Should not exceed threshold at failure {}", i);
    }

    // Record failure that exceeds threshold
    let exceeded = persistence
        .record_failure(task_id, 3)
        .await
        .expect("Record failure");
    assert!(exceeded, "Should exceed threshold");

    let loaded = persistence
        .load_task_state(task_id)
        .await
        .expect("Load state")
        .expect("State exists");
    assert_eq!(loaded.consecutive_failures, Some(3));

    cleanup_task(task_id);
}

#[tokio::test]
async fn test_record_success_resets_counter() {
    dotenvy::dotenv().ok();
    let persistence =
        DatabaseStatePersistence::with_pool_size(2).expect("Failed to create persistence");
    let task_id = "test_success_reset";
    let mut state = setup_test_task(task_id, "test_actor");
    state.consecutive_failures = Some(5);

    persistence
        .save_task_state(task_id, &state)
        .await
        .expect("Save initial state");

    // Record success
    persistence
        .record_success(task_id)
        .await
        .expect("Record success");

    // Check counter reset
    let loaded = persistence
        .load_task_state(task_id)
        .await
        .expect("Load state")
        .expect("State exists");
    assert_eq!(loaded.consecutive_failures, Some(0));

    cleanup_task(task_id);
}

#[tokio::test]
async fn test_should_execute_respects_pause_state() {
    dotenvy::dotenv().ok();
    let persistence =
        DatabaseStatePersistence::with_pool_size(2).expect("Failed to create persistence");
    let task_id = "test_should_execute";
    let state = setup_test_task(task_id, "test_actor");

    persistence
        .save_task_state(task_id, &state)
        .await
        .expect("Save initial state");

    // Should execute when not paused
    let should_run = persistence
        .should_execute(task_id)
        .await
        .expect("Check execution");
    assert!(should_run, "Should execute when not paused");

    // Pause task
    persistence.pause_task(task_id).await.expect("Pause task");

    // Should not execute when paused
    let should_run = persistence
        .should_execute(task_id)
        .await
        .expect("Check execution");
    assert!(!should_run, "Should not execute when paused");

    // Resume task
    persistence.resume_task(task_id).await.expect("Resume task");

    // Should execute after resume
    let should_run = persistence
        .should_execute(task_id)
        .await
        .expect("Check execution");
    assert!(should_run, "Should execute after resume");

    cleanup_task(task_id);
}

#[tokio::test]
async fn test_circuit_breaker_with_execution_logging() {
    dotenvy::dotenv().ok();
    let persistence =
        DatabaseStatePersistence::with_pool_size(2).expect("Failed to create persistence");
    let task_id = "test_circuit_breaker_logging";
    let actor_name = "test_actor";

    cleanup_task(task_id);
    let state = setup_test_task(task_id, actor_name);

    persistence
        .save_task_state(task_id, &state)
        .await
        .expect("Save initial state");

    // Simulate failed execution
    let exec_id = persistence
        .start_execution(task_id, actor_name)
        .await
        .expect("Start execution");

    persistence
        .fail_execution(exec_id, "Test error")
        .await
        .expect("Fail execution");

    // Record failure in circuit breaker
    persistence
        .record_failure(task_id, 3)
        .await
        .expect("Record failure");

    // Check execution history
    let history = persistence
        .get_execution_history(task_id, 10)
        .await
        .expect("Get history");
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].success, Some(false));

    // Simulate successful execution
    let exec_id = persistence
        .start_execution(task_id, actor_name)
        .await
        .expect("Start execution");

    let result = DatabaseExecutionResult {
        skills_succeeded: 5,
        skills_failed: 0,
        skills_skipped: 0,
        metadata: serde_json::json!({}),
    };

    persistence
        .complete_execution(exec_id, result)
        .await
        .expect("Complete execution");

    // Record success - should reset counter
    persistence
        .record_success(task_id)
        .await
        .expect("Record success");

    let loaded = persistence
        .load_task_state(task_id)
        .await
        .expect("Load state")
        .expect("State exists");
    assert_eq!(loaded.consecutive_failures, Some(0));

    cleanup_task(task_id);
}

#[tokio::test]
async fn test_manual_pause_overrides_circuit_breaker() {
    dotenvy::dotenv().ok();
    let persistence =
        DatabaseStatePersistence::with_pool_size(2).expect("Failed to create persistence");
    let task_id = "test_manual_pause";
    let state = setup_test_task(task_id, "test_actor");

    persistence
        .save_task_state(task_id, &state)
        .await
        .expect("Save initial state");

    // Manually pause before failures
    persistence.pause_task(task_id).await.expect("Pause task");

    // Should not execute even with zero failures
    let should_run = persistence
        .should_execute(task_id)
        .await
        .expect("Check execution");
    assert!(!should_run, "Manual pause should prevent execution");

    // Record failures (doesn't matter, already paused)
    persistence
        .record_failure(task_id, 3)
        .await
        .expect("Record failure");

    // Still should not execute
    let should_run = persistence
        .should_execute(task_id)
        .await
        .expect("Check execution");
    assert!(!should_run, "Should remain paused");

    // Resume manually
    persistence.resume_task(task_id).await.expect("Resume task");

    // Now should execute
    let should_run = persistence
        .should_execute(task_id)
        .await
        .expect("Check execution");
    assert!(should_run, "Should execute after manual resume");

    cleanup_task(task_id);
}
