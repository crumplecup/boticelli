use botticelli_actor::DatabaseStatePersistence;
use botticelli_database::{ActorServerStateRow, establish_connection};
use chrono::Utc;
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
async fn test_single_state_save() {
    dotenvy::dotenv().ok();

    let task_id = "minimal_test_single_save";
    cleanup_task(task_id);

    let persistence = match DatabaseStatePersistence::with_pool_size(2) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("✗ Failed to create persistence: {}", e);
            panic!("Failed to create persistence: {}", e);
        }
    };

    let state = create_test_state(task_id, "test_actor");

    match persistence.save_task_state(task_id, &state).await {
        Ok(_) => println!("✓ State saved successfully"),
        Err(e) => {
            eprintln!("✗ Failed to save state: {}", e);
            cleanup_task(task_id);
            panic!("Failed to save state: {}", e);
        }
    }

    cleanup_task(task_id);
}

#[tokio::test]
async fn test_single_state_load() {
    dotenvy::dotenv().ok();

    let task_id = "minimal_test_single_load";
    cleanup_task(task_id);

    let persistence = match DatabaseStatePersistence::with_pool_size(2) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("✗ Failed to create persistence: {}", e);
            panic!("Failed to create persistence: {}", e);
        }
    };

    let state = create_test_state(task_id, "test_load_actor");

    // Save first
    if let Err(e) = persistence.save_task_state(task_id, &state).await {
        eprintln!("✗ Failed to save state: {}", e);
        cleanup_task(task_id);
        panic!("Failed to save state: {}", e);
    }

    // Then load
    match persistence.load_task_state(task_id).await {
        Ok(Some(loaded_state)) => {
            println!("✓ State loaded successfully");
            assert_eq!(loaded_state.task_id, task_id);
            assert_eq!(loaded_state.actor_name, "test_load_actor");
        }
        Ok(None) => {
            eprintln!("✗ No state found");
            cleanup_task(task_id);
            panic!("Expected state to be found");
        }
        Err(e) => {
            eprintln!("✗ Failed to load state: {}", e);
            cleanup_task(task_id);
            panic!("Failed to load state: {}", e);
        }
    }

    cleanup_task(task_id);
}

#[tokio::test]
async fn test_two_task_inserts() {
    dotenvy::dotenv().ok();

    let persistence = match DatabaseStatePersistence::with_pool_size(2) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("✗ Failed to create persistence: {}", e);
            panic!("Failed to create persistence: {}", e);
        }
    };

    // Insert two tasks using a loop
    for i in 0..2 {
        let task_id = format!("minimal_test_task_{}", i);
        let actor_name = format!("actor_{}", i);

        cleanup_task(&task_id);

        let state = create_test_state(&task_id, &actor_name);
        println!("Inserting task {}...", i);
        if let Err(e) = persistence.save_task_state(&task_id, &state).await {
            eprintln!("✗ Failed to insert task {}: {}", i, e);
            cleanup_task(&task_id);
            panic!("Failed to insert task {}: {}", i, e);
        }
        println!("✓ Task {} inserted", i);
    }

    // Load both tasks using a loop
    for i in 0..2 {
        let task_id = format!("minimal_test_task_{}", i);
        let expected_actor = format!("actor_{}", i);

        println!("Loading task {}...", i);
        match persistence.load_task_state(&task_id).await {
            Ok(Some(loaded)) => {
                println!("✓ Task {} found: actor_name={}", i, loaded.actor_name);
                assert_eq!(loaded.actor_name, expected_actor);
            }
            Ok(None) => {
                eprintln!("✗ Task {} NOT FOUND!", i);
                cleanup_task(&task_id);
                panic!("Task {} missing after insert!", i);
            }
            Err(e) => {
                eprintln!("✗ Failed to load task {}: {}", i, e);
                cleanup_task(&task_id);
                panic!("Failed to load task {}: {}", i, e);
            }
        }

        cleanup_task(&task_id);
    }

    println!("✓ Both tasks verified");
}

#[tokio::test]
async fn test_two_task_inserts_competing() {
    dotenvy::dotenv().ok();

    let persistence = match DatabaseStatePersistence::with_pool_size(5) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("✗ Failed to create persistence: {}", e);
            panic!("Failed to create persistence: {}", e);
        }
    };

    // Insert five tasks with different prefix using loop
    for i in 0..5 {
        let task_id = format!("competing_test_task_{}", i);
        let actor_name = format!("competing_actor_{}", i);

        cleanup_task(&task_id);

        let state = create_test_state(&task_id, &actor_name);
        println!("Inserting competing task {}...", i);
        if let Err(e) = persistence.save_task_state(&task_id, &state).await {
            eprintln!("✗ Failed to insert competing task {}: {}", i, e);
            cleanup_task(&task_id);
            panic!("Failed to insert competing task {}: {}", i, e);
        }
        println!("✓ Competing task {} inserted", i);
    }

    // Load all five tasks
    for i in 0..5 {
        let task_id = format!("competing_test_task_{}", i);
        let expected_actor = format!("competing_actor_{}", i);

        println!("Loading competing task {}...", i);
        match persistence.load_task_state(&task_id).await {
            Ok(Some(loaded)) => {
                println!(
                    "✓ Competing task {} found: actor_name={}",
                    i, loaded.actor_name
                );
                assert_eq!(loaded.actor_name, expected_actor);
            }
            Ok(None) => {
                eprintln!("✗ Competing task {} NOT FOUND!", i);
                cleanup_task(&task_id);
                panic!("Competing task {} missing after insert!", i);
            }
            Err(e) => {
                eprintln!("✗ Failed to load competing task {}: {}", i, e);
                cleanup_task(&task_id);
                panic!("Failed to load competing task {}: {}", i, e);
            }
        }

        cleanup_task(&task_id);
    }

    println!("✓ All five competing tasks verified");
}
