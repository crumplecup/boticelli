//! In-memory implementation of NarrativeRepository for testing.
//!
//! This module provides a simple HashMap-based repository that stores executions
//! in memory. Useful for unit tests and demonstrating the trait interface.

use crate::{
    BoticelliError, BoticelliErrorKind, BoticelliResult, ExecutionFilter, ExecutionStatus,
    ExecutionSummary, NarrativeExecution, NarrativeRepository,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[cfg(feature = "database")]
use chrono::Utc;

/// In-memory repository for narrative executions.
///
/// Stores executions in a HashMap protected by an RwLock for thread-safe access.
/// All data is lost when the repository is dropped.
///
/// # Example
/// ```no_run
/// use boticelli::{InMemoryNarrativeRepository, NarrativeRepository};
///
/// #[tokio::main]
/// async fn main() {
///     let repo = InMemoryNarrativeRepository::new();
///     // Use repo.save_execution(), load_execution(), etc.
/// }
/// ```
#[derive(Debug, Clone)]
pub struct InMemoryNarrativeRepository {
    /// Storage for executions, keyed by ID
    executions: Arc<RwLock<HashMap<i32, StoredExecution>>>,
    /// Next ID to assign
    next_id: Arc<RwLock<i32>>,
}

/// Internal storage structure for executions.
#[derive(Debug, Clone)]
struct StoredExecution {
    id: i32,
    narrative_name: String,
    narrative_description: Option<String>,
    status: ExecutionStatus,
    #[cfg(feature = "database")]
    started_at: chrono::DateTime<Utc>,
    #[cfg(feature = "database")]
    completed_at: Option<chrono::DateTime<Utc>>,
    execution: NarrativeExecution,
    error_message: Option<String>,
}

impl InMemoryNarrativeRepository {
    /// Create a new empty in-memory repository.
    pub fn new() -> Self {
        Self {
            executions: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
        }
    }

    /// Get the number of stored executions (for testing).
    pub async fn len(&self) -> usize {
        self.executions.read().await.len()
    }

    /// Check if the repository is empty (for testing).
    pub async fn is_empty(&self) -> bool {
        self.executions.read().await.is_empty()
    }

    /// Clear all executions (for testing).
    pub async fn clear(&self) {
        self.executions.write().await.clear();
        *self.next_id.write().await = 1;
    }
}

impl Default for InMemoryNarrativeRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NarrativeRepository for InMemoryNarrativeRepository {
    async fn save_execution(&self, execution: &NarrativeExecution) -> BoticelliResult<i32> {
        let mut next_id_guard = self.next_id.write().await;
        let id = *next_id_guard;
        *next_id_guard += 1;
        drop(next_id_guard);

        let stored = StoredExecution {
            id,
            narrative_name: execution.narrative_name.clone(),
            narrative_description: None, // Not available in current NarrativeExecution
            status: ExecutionStatus::Completed,
            #[cfg(feature = "database")]
            started_at: Utc::now(),
            #[cfg(feature = "database")]
            completed_at: Some(Utc::now()),
            execution: execution.clone(),
            error_message: None,
        };

        self.executions.write().await.insert(id, stored);
        Ok(id)
    }

    async fn load_execution(&self, id: i32) -> BoticelliResult<NarrativeExecution> {
        let executions = self.executions.read().await;
        executions
            .get(&id)
            .map(|stored| stored.execution.clone())
            .ok_or_else(|| {
                BoticelliError::new(BoticelliErrorKind::Backend(format!(
                    "Execution {} not found",
                    id
                )))
            })
    }

    async fn list_executions(
        &self,
        filter: &ExecutionFilter,
    ) -> BoticelliResult<Vec<ExecutionSummary>> {
        let executions = self.executions.read().await;
        let mut results: Vec<ExecutionSummary> = executions
            .values()
            .filter(|stored| {
                // Apply narrative_name filter
                if let Some(ref name) = filter.narrative_name
                    && &stored.narrative_name != name
                {
                    return false;
                }

                // Apply status filter
                if let Some(ref status) = filter.status
                    && &stored.status != status
                {
                    return false;
                }

                // Apply date range filters (only with database feature)
                #[cfg(feature = "database")]
                {
                    if let Some(ref after) = filter.started_after
                        && &stored.started_at < after
                    {
                        return false;
                    }

                    if let Some(ref before) = filter.started_before
                        && &stored.started_at > before
                    {
                        return false;
                    }
                }

                true
            })
            .map(|stored| ExecutionSummary {
                id: stored.id,
                narrative_name: stored.narrative_name.clone(),
                narrative_description: stored.narrative_description.clone(),
                status: stored.status,
                #[cfg(feature = "database")]
                started_at: stored.started_at,
                #[cfg(feature = "database")]
                completed_at: stored.completed_at,
                act_count: stored.execution.act_executions.len(),
                error_message: stored.error_message.clone(),
            })
            .collect();

        // Sort by ID for consistent ordering
        results.sort_by_key(|s| s.id);

        // Apply pagination
        let offset = filter.offset.unwrap_or(0);
        let limit = filter.limit.unwrap_or(usize::MAX);

        Ok(results.into_iter().skip(offset).take(limit).collect())
    }

    async fn update_status(&self, id: i32, status: ExecutionStatus) -> BoticelliResult<()> {
        let mut executions = self.executions.write().await;
        executions
            .get_mut(&id)
            .map(|stored| {
                stored.status = status;
                #[cfg(feature = "database")]
                if matches!(status, ExecutionStatus::Completed | ExecutionStatus::Failed) {
                    stored.completed_at = Some(Utc::now());
                }
            })
            .ok_or_else(|| {
                BoticelliError::new(BoticelliErrorKind::Backend(format!(
                    "Execution {} not found",
                    id
                )))
            })
    }

    async fn delete_execution(&self, id: i32) -> BoticelliResult<()> {
        self.executions
            .write()
            .await
            .remove(&id)
            .map(|_| ())
            .ok_or_else(|| {
                BoticelliError::new(BoticelliErrorKind::Backend(format!(
                    "Execution {} not found",
                    id
                )))
            })
    }

    // Video methods use default implementations (return NotImplemented)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ActExecution, Input};

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
            .store_video(
                b"fake_video_data",
                &crate::VideoMetadata::new(b"fake_video_data"),
            )
            .await;
        assert!(store_result.is_err());

        let load_result = repo.load_video("fake_ref").await;
        assert!(load_result.is_err());
    }
}
