//! Integration tests for task scheduler with database persistence.

use botticelli_actor::{DatabaseStatePersistence, SimpleTaskScheduler};
use botticelli_server::{ActorServerResult, TaskScheduler};
use std::time::Duration;

#[tokio::test]
async fn test_scheduler_with_persistence() -> ActorServerResult<()> {
    dotenvy::dotenv().ok();
    let persistence =
        DatabaseStatePersistence::with_pool_size(2).expect("Failed to create persistence");
    let mut scheduler = SimpleTaskScheduler::with_persistence(persistence);

    assert!(scheduler.has_persistence());

    // Schedule a simple task
    scheduler
        .schedule("test_task".to_string(), Duration::from_secs(10), || async {
            Ok(())
        })
        .await?;

    assert!(scheduler.is_scheduled("test_task"));

    // Cancel the task
    scheduler.cancel("test_task").await?;

    assert!(!scheduler.is_scheduled("test_task"));

    Ok(())
}

#[tokio::test]
async fn test_scheduler_without_persistence() -> ActorServerResult<()> {
    let mut scheduler = SimpleTaskScheduler::new();

    assert!(!scheduler.has_persistence());

    // Schedule a simple task
    scheduler
        .schedule("test_task".to_string(), Duration::from_secs(10), || async {
            Ok(())
        })
        .await?;

    assert!(scheduler.is_scheduled("test_task"));

    // Cancel the task
    scheduler.cancel("test_task").await?;

    Ok(())
}

#[tokio::test]
async fn test_scheduler_task_recovery() -> ActorServerResult<()> {
    dotenvy::dotenv().ok();
    let persistence =
        DatabaseStatePersistence::with_pool_size(2).expect("Failed to create persistence");
    let scheduler = SimpleTaskScheduler::with_persistence(persistence);

    // Attempt recovery (should handle empty state gracefully)
    let recovered = scheduler.recover_tasks().await?;

    assert_eq!(recovered.len(), 0);

    Ok(())
}
