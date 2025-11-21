//! Comprehensive tests for botticelli_security crate.

use botticelli_security::*;
use std::collections::HashMap;

// ============================================================================
// Rate Limiter Tests
// ============================================================================

#[test]
fn test_rate_limit_allows_multiple() {
    let mut limiter = RateLimiter::new();
    let limit = RateLimit::strict(5, 60);
    limiter.add_limit("test.operation", limit);

    // First 5 should pass
    for _ in 0..5 {
        assert!(limiter.check("test.operation").is_ok());
    }
}

#[test]
fn test_rate_limit_blocks_excess() {
    let mut limiter = RateLimiter::new();
    let limit = RateLimit::strict(2, 60);
    limiter.add_limit("test.operation", limit);

    // First 2 should pass
    limiter.check("test.operation").unwrap();
    limiter.check("test.operation").unwrap();

    // Third should fail
    assert!(limiter.check("test.operation").is_err());
}

#[test]
fn test_rate_limit_with_burst() {
    let mut limiter = RateLimiter::new();
    let limit = RateLimit::new(5, 60, 2); // 5 per minute + 2 burst
    limiter.add_limit("test.operation", limit);

    // Should allow 7 requests (5 + 2 burst)
    for _ in 0..7 {
        assert!(limiter.check("test.operation").is_ok());
    }

    // 8th should fail
    assert!(limiter.check("test.operation").is_err());
}

#[test]
fn test_rate_limit_unconfigured_allows() {
    let mut limiter = RateLimiter::new();
    
    // No limit configured, should always pass
    assert!(limiter.check("unconfigured.operation").is_ok());
    assert!(limiter.check("unconfigured.operation").is_ok());
}

#[test]
fn test_rate_limit_available_tokens() {
    let mut limiter = RateLimiter::new();
    let limit = RateLimit::strict(5, 60);
    limiter.add_limit("test.operation", limit);

    // Should have 5 tokens initially
    assert_eq!(limiter.available_tokens("test.operation"), Some(5));

    // Consume one
    limiter.check("test.operation").unwrap();
    assert_eq!(limiter.available_tokens("test.operation"), Some(4));
}

// ============================================================================
// Content Filter Tests
// ============================================================================

#[test]
fn test_content_filter_clean_text() {
    let config = ContentFilterConfig::default();
    let filter = ContentFilter::new(config).unwrap();
    assert!(filter.filter("This is clean text").is_ok());
}

#[test]
fn test_content_filter_max_length() {
    let config = ContentFilterConfig::new().with_max_length(10);
    let filter = ContentFilter::new(config).unwrap();
    
    assert!(filter.filter("Short").is_ok());
    assert!(filter.filter("This is way too long for the limit").is_err());
}

#[test]
fn test_content_filter_custom_pattern() {
    let patterns = vec!["\\b(secret|password)\\b".to_string()];
    let config = ContentFilterConfig::new().with_prohibited_patterns(patterns);
    let filter = ContentFilter::new(config).unwrap();
    
    assert!(filter.filter("Safe content here").is_ok());
    assert!(filter.filter("The secret code is 123").is_err());
    assert!(filter.filter("My password is hunter2").is_err());
}

#[test]
fn test_content_filter_mass_mentions() {
    let config = ContentFilterConfig::default();
    let filter = ContentFilter::new(config).unwrap();
    
    // @everyone and @here should be blocked
    assert!(filter.filter("Hey @everyone look at this!").is_err());
    assert!(filter.filter("Attention @here please").is_err());
}

// ============================================================================
// Permission Checker Tests
// ============================================================================

#[test]
fn test_permission_default_deny() {
    let config = PermissionConfig::default();
    let checker = PermissionChecker::new(config);
    let result = checker.check_command("test.command");
    assert!(result.is_err());
}

#[test]
fn test_permission_allow_by_default() {
    let config = PermissionConfig::new()
        .with_allow_all_by_default(true);
    let checker = PermissionChecker::new(config);
    let result = checker.check_command("test.command");
    assert!(result.is_ok());
}

// ============================================================================
// Approval Workflow Tests
// ============================================================================

#[test]
fn test_approval_create_action() {
    let mut workflow = ApprovalWorkflow::new();
    let params = HashMap::new();
    
    let action_id = workflow.create_pending_action(
        "narrative1",
        "test.command",
        params,
        None,
    ).unwrap();
    
    assert!(workflow.get_pending_action(&action_id).is_some());
}

#[test]
fn test_approval_approve_action() {
    let mut workflow = ApprovalWorkflow::new();
    let params = HashMap::new();
    
    let action_id = workflow.create_pending_action(
        "narrative1",
        "test.command",
        params,
        None,
    ).unwrap();
    
    workflow.approve_action(&action_id, "admin", Some("Looks good".to_string())).unwrap();
    
    assert!(workflow.check_approval(&action_id).is_ok());
}

#[test]
fn test_approval_deny_action() {
    let mut workflow = ApprovalWorkflow::new();
    let params = HashMap::new();
    
    let action_id = workflow.create_pending_action(
        "narrative1",
        "test.command",
        params,
        None,
    ).unwrap();
    
    workflow.deny_action(&action_id, "admin", Some("Not allowed".to_string())).unwrap();
    
    assert!(workflow.check_approval(&action_id).is_err());
}

#[test]
fn test_approval_pending_blocks() {
    let mut workflow = ApprovalWorkflow::new();
    let params = HashMap::new();
    
    let action_id = workflow.create_pending_action(
        "narrative1",
        "test.command",
        params,
        None,
    ).unwrap();
    
    // Should block while pending
    assert!(workflow.check_approval(&action_id).is_err());
}

// ============================================================================
// Validation Tests
// ============================================================================

#[test]
fn test_validation_error_creation() {
    let error = ValidationError::new("test_field", "Invalid value");
    assert_eq!(error.field, "test_field");
    assert_eq!(error.reason, "Invalid value");
}
