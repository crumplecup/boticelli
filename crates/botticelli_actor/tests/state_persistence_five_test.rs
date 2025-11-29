use botticelli_actor::DatabaseStatePersistence;
use botticelli_database::{ActorServerStateRow, establish_connection};
use chrono::Utc;
use diesel::prelude::*;

fn cleanup_tasks() {
    let mut conn = establish_connection().expect("Database connection");
    for i in 0..5 {
        let task_id = format!("five_test_task_{}", i);
        diesel::delete(
            botticelli_database::schema::actor_server_state::table
                .filter(botticelli_database::schema::actor_server_state::task_id.eq(&task_id)),
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
async fn test_five_task_inserts() {
    dotenvy::dotenv().ok();

    cleanup_tasks();

    let persistence = match DatabaseStatePersistence::with_pool_size(5) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("✗ Failed to create persistence: {}", e);
            panic!("Failed to create persistence: {}", e);
        }
    };

    // Insert 5 tasks
    for i in 0..5 {
        let task_id = format!("five_test_task_{}", i);
        let actor_name = format!("actor_{}", i);
        let state = create_test_state(&task_id, &actor_name);

        println!("Inserting task {}...", i);
        if let Err(e) = persistence.save_task_state(&task_id, &state).await {
            eprintln!("✗ Failed to insert task {}: {}", i, e);
            cleanup_tasks();
            panic!("Failed to insert task {}: {}", i, e);
        }
        println!("✓ Task {} inserted", i);
    }

    // Load all 5 tasks
    println!("Verifying all 5 tasks...");
    for i in 0..5 {
        let task_id = format!("five_test_task_{}", i);
        let expected_actor = format!("actor_{}", i);

        println!("Loading task {}...", i);
        match persistence.load_task_state(&task_id).await {
            Ok(Some(loaded)) => {
                println!("✓ Task {} found: actor_name={}", i, loaded.actor_name);
                assert_eq!(loaded.actor_name, expected_actor);
            }
            Ok(None) => {
                eprintln!("✗ Task {} NOT FOUND!", i);
                cleanup_tasks();
                panic!("Task {} missing after insert!", i);
            }
            Err(e) => {
                eprintln!("✗ Failed to load task {}: {}", i, e);
                cleanup_tasks();
                panic!("Failed to load task {}: {}", i, e);
            }
        }
    }

    println!("✓ All 5 tasks verified");
    cleanup_tasks();
}
