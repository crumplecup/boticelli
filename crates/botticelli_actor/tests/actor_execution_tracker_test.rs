//! Tests for actor execution tracker integration

use botticelli_actor::{ActorExecutionTracker, DatabaseExecutionResult, DatabaseStatePersistence};
use botticelli_database::{NewActorServerStateBuilder, establish_connection};
use chrono::{Duration, Utc};
use diesel::RunQueryDsl;
use std::sync::Arc;

#[tokio::test]
async fn test_execution_tracker_lifecycle() {
    dotenvy::dotenv().ok();
    let persistence = Arc::new(
        DatabaseStatePersistence::with_pool_size(2).expect("Failed to create persistence"),
    );
    let task_id = format!(
        "test-tracker-{}-{}",
        Utc::now().timestamp_millis(),
        std::process::id()
    );
    let actor_name = "test-actor";

    // Setup: Create initial state
    {
        let mut conn = establish_connection().expect("Database connection");
        let new_state = NewActorServerStateBuilder::default()
            .task_id(task_id.clone())
            .actor_name(actor_name.to_string())
            .next_run(Utc::now().naive_utc())
            .is_paused(false)
            .consecutive_failures(0)
            .metadata(serde_json::json!({}))
            .build()
            .expect("Valid state");

        diesel::insert_into(botticelli_database::schema::actor_server_state::table)
            .values(&new_state)
            .execute(&mut conn)
            .expect("Insert state");
    }

    let tracker =
        ActorExecutionTracker::new(persistence.clone(), task_id.clone(), actor_name.to_string());

    // Should execute initially
    assert!(
        tracker.should_execute().await.expect("Check execution"),
        "Task should be executable initially"
    );

    // Start execution
    let exec_id = tracker.start_execution().await.expect("Start execution");
    assert!(exec_id > 0, "Execution ID should be positive");

    // Record success
    let result = DatabaseExecutionResult {
        skills_succeeded: 3,
        skills_failed: 0,
        skills_skipped: 1,
        metadata: serde_json::json!({"test": "success"}),
    };

    tracker
        .record_success(exec_id, result)
        .await
        .expect("Record success");

    // Update next run
    let next_run = (Utc::now() + Duration::seconds(60)).naive_utc();
    tracker
        .update_next_run(next_run)
        .await
        .expect("Update next run");

    // Cleanup
    persistence
        .delete_task_state(&task_id)
        .await
        .expect("Cleanup");
}

#[tokio::test]
async fn test_execution_tracker_circuit_breaker() {
    dotenvy::dotenv().ok();
    let persistence = Arc::new(
        DatabaseStatePersistence::with_pool_size(2).expect("Failed to create persistence"),
    );
    let task_id = format!(
        "test-circuit-{}-{}",
        Utc::now().timestamp_millis(),
        std::process::id()
    );
    let actor_name = "failing-actor";

    // Setup with max_failures = 3 in metadata
    {
        let mut conn = establish_connection().expect("Database connection");
        let new_state = NewActorServerStateBuilder::default()
            .task_id(task_id.clone())
            .actor_name(actor_name.to_string())
            .next_run(Utc::now().naive_utc())
            .is_paused(false)
            .consecutive_failures(0)
            .metadata(serde_json::json!({"max_failures": 3}))
            .build()
            .expect("Valid state");

        diesel::insert_into(botticelli_database::schema::actor_server_state::table)
            .values(&new_state)
            .execute(&mut conn)
            .expect("Insert state");
    }

    let tracker =
        ActorExecutionTracker::new(persistence.clone(), task_id.clone(), actor_name.to_string());

    // Record 3 failures
    for i in 1..=3 {
        let exec_id = tracker.start_execution().await.expect("Start execution");
        let should_pause = tracker
            .record_failure(exec_id, &format!("Error {}", i))
            .await
            .expect("Record failure");

        if i < 3 {
            assert!(!should_pause, "Should not pause until threshold exceeded");
        } else {
            assert!(should_pause, "Should pause after 3 failures");
        }
    }

    // Task should now be paused
    assert!(
        !tracker.should_execute().await.expect("Check execution"),
        "Task should be paused after circuit breaker"
    );

    // Cleanup
    persistence
        .delete_task_state(&task_id)
        .await
        .expect("Cleanup");
}

#[tokio::test]
async fn test_execution_tracker_accessors() {
    dotenvy::dotenv().ok();
    let persistence = Arc::new(
        DatabaseStatePersistence::with_pool_size(2).expect("Failed to create persistence"),
    );
    let task_id = "test-task".to_string();
    let actor_name = "test-actor".to_string();

    let tracker =
        ActorExecutionTracker::new(persistence.clone(), task_id.clone(), actor_name.clone());

    assert_eq!(tracker.task_id(), "test-task");
    assert_eq!(tracker.actor_name(), "test-actor");
    assert!(Arc::ptr_eq(tracker.persistence(), &persistence));
}
