//! Multi-task state persistence tests.

use botticelli_actor::DatabaseStatePersistence;
use botticelli_database::{ActorServerStateRow, establish_connection};
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;

fn cleanup_task(task_id: &str) {
    let mut conn = establish_connection().expect("Database connection");
    diesel::delete(
        botticelli_database::schema::actor_server_state::table
            .filter(botticelli_database::schema::actor_server_state::task_id.eq(task_id)),
    )
    .execute(&mut conn)
    .ok();
}

fn cleanup_tasks(task_ids: &[&str]) {
    let mut conn = establish_connection().expect("Database connection");
    for task_id in task_ids {
        diesel::delete(
            botticelli_database::schema::actor_server_state::table
                .filter(botticelli_database::schema::actor_server_state::task_id.eq(task_id)),
        )
        .execute(&mut conn)
        .ok();
    }
}

fn create_test_state(task_id: &str, actor_name: &str) -> ActorServerStateRow {
    let now = Utc::now().naive_utc();
    ActorServerStateRow {
        task_id: task_id.to_string(),
        actor_name: actor_name.to_string(),
        last_run: None,
        next_run: now,
        consecutive_failures: Some(0),
        is_paused: Some(false),
        metadata: Some(serde_json::json!({})),
        updated_at: now,
    }
}

#[tokio::test]
async fn test_save_and_load_task_state() {
    dotenvy::dotenv().ok();
    dotenvy::dotenv().ok();
    let task_id = "test_save_and_load_task_state";
    cleanup_task(task_id);

    let persistence =
        DatabaseStatePersistence::with_pool_size(2).expect("Failed to create persistence");
    let state = create_test_state(task_id, "actor1");

    persistence
        .save_task_state(task_id, &state)
        .await
        .expect("Save failed");

    let loaded = persistence
        .load_task_state(task_id)
        .await
        .expect("Load failed")
        .expect("State not found");

    assert_eq!(loaded.task_id, task_id);
    assert_eq!(loaded.actor_name, "actor1");
    assert_eq!(loaded.consecutive_failures, Some(0));
    assert_eq!(loaded.is_paused, Some(false));

    cleanup_task(task_id);
}

#[tokio::test]
async fn test_load_nonexistent_task() {
    dotenvy::dotenv().ok();
    let persistence =
        DatabaseStatePersistence::with_pool_size(2).expect("Failed to create persistence");

    let loaded = persistence
        .load_task_state("nonexistent_task_xyz")
        .await
        .expect("Load failed");

    assert!(loaded.is_none());
}

#[tokio::test]
async fn test_delete_task_state() {
    dotenvy::dotenv().ok();
    let task_id = "test_delete_task_state";
    cleanup_task(task_id);

    let persistence =
        DatabaseStatePersistence::with_pool_size(2).expect("Failed to create persistence");
    let state = create_test_state(task_id, "actor1");

    persistence
        .save_task_state(task_id, &state)
        .await
        .expect("Save failed");

    persistence
        .delete_task_state(task_id)
        .await
        .expect("Delete failed");

    let loaded = persistence
        .load_task_state(task_id)
        .await
        .expect("Load failed");

    assert!(loaded.is_none());
}

#[tokio::test]
async fn test_list_all_tasks() {
    dotenvy::dotenv().ok();
    let task_ids = [
        "test_list_all_tasks_1",
        "test_list_all_tasks_2",
        "test_list_all_tasks_3",
    ];
    cleanup_tasks(&task_ids);

    let persistence =
        DatabaseStatePersistence::with_pool_size(2).expect("Failed to create persistence");

    let state1 = create_test_state(task_ids[0], "actor1");
    let state2 = create_test_state(task_ids[1], "actor2");
    let state3 = create_test_state(task_ids[2], "actor1");

    persistence
        .save_task_state(task_ids[0], &state1)
        .await
        .expect("Save failed");
    persistence
        .save_task_state(task_ids[1], &state2)
        .await
        .expect("Save failed");
    persistence
        .save_task_state(task_ids[2], &state3)
        .await
        .expect("Save failed");

    let tasks = persistence.list_all_tasks().await.expect("List failed");

    assert!(tasks.len() >= 3);
    assert!(tasks.iter().any(|t| t.task_id == task_ids[0]));
    assert!(tasks.iter().any(|t| t.task_id == task_ids[1]));
    assert!(tasks.iter().any(|t| t.task_id == task_ids[2]));

    cleanup_tasks(&task_ids);
}

#[tokio::test]
async fn test_list_tasks_by_actor() {
    dotenvy::dotenv().ok();
    let task_ids = [
        "test_list_tasks_by_actor_1",
        "test_list_tasks_by_actor_2",
        "test_list_tasks_by_actor_3",
    ];
    cleanup_tasks(&task_ids);

    let persistence =
        DatabaseStatePersistence::with_pool_size(2).expect("Failed to create persistence");

    let state1 = create_test_state(task_ids[0], "test_actor_unique_1");
    let state2 = create_test_state(task_ids[1], "test_actor_unique_2");
    let state3 = create_test_state(task_ids[2], "test_actor_unique_1");

    persistence
        .save_task_state(task_ids[0], &state1)
        .await
        .expect("Save failed");
    persistence
        .save_task_state(task_ids[1], &state2)
        .await
        .expect("Save failed");
    persistence
        .save_task_state(task_ids[2], &state3)
        .await
        .expect("Save failed");

    let tasks = persistence
        .list_tasks_by_actor("test_actor_unique_1")
        .await
        .expect("List failed");

    assert_eq!(tasks.len(), 2);
    assert!(tasks.iter().all(|t| t.actor_name == "test_actor_unique_1"));

    cleanup_tasks(&task_ids);
}

#[tokio::test]
async fn test_list_active_and_paused_tasks() {
    dotenvy::dotenv().ok();
    let task_ids = [
        "test_list_active_paused_1",
        "test_list_active_paused_2",
        "test_list_active_paused_3",
    ];
    cleanup_tasks(&task_ids);

    let persistence =
        DatabaseStatePersistence::with_pool_size(2).expect("Failed to create persistence");

    let mut state1 = create_test_state(task_ids[0], "actor1");
    state1.is_paused = Some(false);

    let mut state2 = create_test_state(task_ids[1], "actor2");
    state2.is_paused = Some(true);

    let mut state3 = create_test_state(task_ids[2], "actor3");
    state3.is_paused = Some(false);

    persistence
        .save_task_state(task_ids[0], &state1)
        .await
        .expect("Save failed");
    persistence
        .save_task_state(task_ids[1], &state2)
        .await
        .expect("Save failed");
    persistence
        .save_task_state(task_ids[2], &state3)
        .await
        .expect("Save failed");

    let active = persistence.list_active_tasks().await.expect("List failed");
    assert!(active.iter().any(|t| t.task_id == task_ids[0]));
    assert!(active.iter().any(|t| t.task_id == task_ids[2]));

    let paused = persistence.list_paused_tasks().await.expect("List failed");
    assert!(paused.iter().any(|t| t.task_id == task_ids[1]));

    cleanup_tasks(&task_ids);
}

#[tokio::test]
async fn test_pause_and_resume_task() {
    dotenvy::dotenv().ok();
    let task_id = "test_pause_and_resume_task";
    cleanup_task(task_id);

    let persistence =
        DatabaseStatePersistence::with_pool_size(2).expect("Failed to create persistence");
    let state = create_test_state(task_id, "actor1");

    persistence
        .save_task_state(task_id, &state)
        .await
        .expect("Save failed");

    persistence.pause_task(task_id).await.expect("Pause failed");

    let paused_state = persistence
        .load_task_state(task_id)
        .await
        .expect("Load failed")
        .expect("State not found");
    assert_eq!(paused_state.is_paused, Some(true));

    persistence
        .resume_task(task_id)
        .await
        .expect("Resume failed");

    let resumed_state = persistence
        .load_task_state(task_id)
        .await
        .expect("Load failed")
        .expect("State not found");
    assert_eq!(resumed_state.is_paused, Some(false));

    cleanup_task(task_id);
}

#[tokio::test]
async fn test_update_next_run() {
    dotenvy::dotenv().ok();
    let task_id = "test_update_next_run";
    cleanup_task(task_id);

    let persistence =
        DatabaseStatePersistence::with_pool_size(2).expect("Failed to create persistence");
    let state = create_test_state(task_id, "actor1");

    persistence
        .save_task_state(task_id, &state)
        .await
        .expect("Save failed");

    let new_next_run =
        NaiveDateTime::parse_from_str("2025-12-31 23:59:59", "%Y-%m-%d %H:%M:%S").unwrap();

    persistence
        .update_next_run(task_id, new_next_run)
        .await
        .expect("Update failed");

    let updated_state = persistence
        .load_task_state(task_id)
        .await
        .expect("Load failed")
        .expect("State not found");
    assert_eq!(updated_state.next_run, new_next_run);

    cleanup_task(task_id);
}

#[tokio::test]
async fn test_concurrent_operations() {
    dotenvy::dotenv().ok();
    let task_ids: Vec<String> = (0..10)
        .map(|i| format!("test_concurrent_operations_{}", i))
        .collect();

    let task_id_refs: Vec<&str> = task_ids.iter().map(|s| s.as_str()).collect();
    cleanup_tasks(&task_id_refs);

    let persistence =
        DatabaseStatePersistence::with_pool_size(2).expect("Failed to create persistence");

    let mut handles = vec![];

    for (i, task_id) in task_ids.iter().enumerate() {
        let p = persistence.clone();
        let task_id = task_id.clone();
        let actor_name = format!("actor{}", i % 3);

        let handle = tokio::spawn(async move {
            let state = create_test_state(&task_id, &actor_name);
            p.save_task_state(&task_id, &state)
                .await
                .expect("Save failed");
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.expect("Task failed");
    }

    for task_id in &task_ids {
        let loaded = persistence
            .load_task_state(task_id)
            .await
            .expect("Load failed");
        assert!(loaded.is_some(), "Task {} not found", task_id);
    }

    cleanup_tasks(&task_id_refs);
}
