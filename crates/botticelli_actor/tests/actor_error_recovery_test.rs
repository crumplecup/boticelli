//! Tests for actor error recovery and resilience.

use botticelli_actor::{ActorError, ActorErrorKind, ExecutionResultBuilder};

#[test]
fn test_execution_result_tracks_failures() {
    // Test that ExecutionResult correctly tracks failed skill executions
    let result = ExecutionResultBuilder::default()
        .failed(vec![(
            "failing_skill".to_string(),
            ActorError::new(ActorErrorKind::PlatformTemporary("timeout".to_string())),
        )])
        .build()
        .expect("Valid execution result");

    assert_eq!(result.failed.len(), 1);
    assert_eq!(result.succeeded.len(), 0);
    assert_eq!(result.skipped.len(), 0);
}

#[test]
fn test_execution_result_tracks_multiple_states() {
    // Test that ExecutionResult can track succeeded, failed, and skipped skills
    let result = ExecutionResultBuilder::default()
        .succeeded(vec![])
        .failed(vec![(
            "failed_skill".to_string(),
            ActorError::new(ActorErrorKind::ValidationFailed("bad input".to_string())),
        )])
        .skipped(vec!["skipped_skill".to_string()])
        .build()
        .expect("Valid execution result");

    assert_eq!(result.failed.len(), 1);
    assert_eq!(result.skipped.len(), 1);
    assert!(result.succeeded.is_empty());
}

#[test]
fn test_recoverable_errors_allow_continuation() {
    // Test that recoverable errors are properly classified
    let recoverable_errors = vec![
        ActorError::new(ActorErrorKind::PlatformTemporary("timeout".to_string())),
        ActorError::new(ActorErrorKind::RateLimitExceeded(60)),
        ActorError::new(ActorErrorKind::ValidationFailed("retry".to_string())),
        ActorError::new(ActorErrorKind::ResourceUnavailable("busy".to_string())),
    ];

    for error in recoverable_errors {
        assert!(
            error.is_recoverable(),
            "Error {:?} should be recoverable",
            error
        );
    }
}

#[test]
fn test_unrecoverable_errors_stop_execution() {
    // Test that unrecoverable errors are properly classified
    let unrecoverable_errors = vec![
        ActorError::new(ActorErrorKind::AuthenticationFailed("invalid".to_string())),
        ActorError::new(ActorErrorKind::InvalidConfiguration("bad".to_string())),
        ActorError::new(ActorErrorKind::PlatformPermanent("gone".to_string())),
        ActorError::new(ActorErrorKind::DatabaseFailed("connection".to_string())),
    ];

    for error in unrecoverable_errors {
        assert!(
            !error.is_recoverable(),
            "Error {:?} should be unrecoverable",
            error
        );
    }
}
