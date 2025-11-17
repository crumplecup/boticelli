//! Tests for in-memory narrative repository.

use botticelli::{
    ActExecution, ExecutionFilter, ExecutionStatus, InMemoryNarrativeRepository, Input,
    NarrativeExecution, NarrativeRepository, VideoMetadata,
};

fn create_test_execution(name: &str, act_count: usize) -> NarrativeExecution {
    NarrativeExecution {
        narrative_name: name.to_string(),
        act_executions: (0..act_count)
            .map(|i| ActExecution {
                act_name: format!("act_{}", i),
                inputs: vec![Input::Text(format!("Test input {}", i))],
                model: None,
                temperature: None,
                max_tokens: None,
                response: format!("Test response {}", i),
                sequence_number: i,
            })
            .collect(),
    }
}

#[tokio::test]
async fn test_save_and_load() {
    let repo = InMemoryNarrativeRepository::new();
    let execution = create_test_execution("test_narrative", 3);

    let id = repo.save_execution(&execution).await.unwrap();
    assert_eq!(id, 1);

    let loaded = repo.load_execution(id).await.unwrap();
    assert_eq!(loaded.narrative_name, "test_narrative");
    assert_eq!(loaded.act_executions.len(), 3);
}

#[tokio::test]
async fn test_load_not_found() {
    let repo = InMemoryNarrativeRepository::new();
    let result = repo.load_execution(999).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_list_executions() {
    let repo = InMemoryNarrativeRepository::new();

    repo.save_execution(&create_test_execution("narrative_a", 2))
        .await
        .unwrap();
    repo.save_execution(&create_test_execution("narrative_b", 3))
        .await
        .unwrap();
    repo.save_execution(&create_test_execution("narrative_a", 1))
        .await
        .unwrap();

    // List all
    let all = repo.list_executions(&ExecutionFilter::new()).await.unwrap();
    assert_eq!(all.len(), 3);

    // Filter by name
    let filtered = repo
        .list_executions(&ExecutionFilter::new().with_narrative_name("narrative_a"))
        .await
        .unwrap();
    assert_eq!(filtered.len(), 2);

    // Pagination
    let paginated = repo
        .list_executions(&ExecutionFilter::new().with_limit(2))
        .await
        .unwrap();
    assert_eq!(paginated.len(), 2);
}

#[tokio::test]
async fn test_update_status() {
    let repo = InMemoryNarrativeRepository::new();
    let execution = create_test_execution("test", 1);
    let id = repo.save_execution(&execution).await.unwrap();

    repo.update_status(id, ExecutionStatus::Running)
        .await
        .unwrap();

    let summaries = repo.list_executions(&ExecutionFilter::new()).await.unwrap();
    assert_eq!(summaries[0].status, ExecutionStatus::Running);
}

#[tokio::test]
async fn test_delete_execution() {
    let repo = InMemoryNarrativeRepository::new();
    let execution = create_test_execution("test", 1);
    let id = repo.save_execution(&execution).await.unwrap();

    assert_eq!(repo.len().await, 1);

    repo.delete_execution(id).await.unwrap();

    assert_eq!(repo.len().await, 0);
    assert!(repo.load_execution(id).await.is_err());
}

#[tokio::test]
async fn test_clear() {
    let repo = InMemoryNarrativeRepository::new();
    repo.save_execution(&create_test_execution("test1", 1))
        .await
        .unwrap();
    repo.save_execution(&create_test_execution("test2", 1))
        .await
        .unwrap();

    assert_eq!(repo.len().await, 2);

    repo.clear().await;

    assert_eq!(repo.len().await, 0);
}

#[tokio::test]
async fn test_video_not_implemented() {
    let repo = InMemoryNarrativeRepository::new();

    let store_result = repo
        .store_video(b"fake_video_data", &VideoMetadata::new(b"fake_video_data"))
        .await;
    assert!(store_result.is_err());

    let load_result = repo.load_video("fake_ref").await;
    assert!(load_result.is_err());
}
