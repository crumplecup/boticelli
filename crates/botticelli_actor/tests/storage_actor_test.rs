//! Tests for StorageActor error handling and edge cases

use botticelli_actor::{Actor, StorageActor};
use botticelli_database::create_pool;
use serde_json::json;

#[tokio::test]
async fn test_storage_actor_malformed_json() -> anyhow::Result<()> {
    let pool = create_pool()?;
    let actor = StorageActor::new("test_malformed", json!({"invalid": "schema"}));

    // Should handle malformed JSON gracefully
    let result = actor.execute(&pool).await;
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_storage_actor_missing_table() -> anyhow::Result<()> {
    let pool = create_pool()?;
    let actor = StorageActor::new("nonexistent_table_xyz", json!({"test": "data"}));

    // Should create table if missing
    let result = actor.execute(&pool).await;
    assert!(result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_storage_actor_empty_data() -> anyhow::Result<()> {
    let pool = create_pool()?;
    let actor = StorageActor::new("test_empty", json!({}));

    let result = actor.execute(&pool).await;
    assert!(result.is_ok());

    Ok(())
}
