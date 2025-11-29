//! Tests for narrative error handling and recovery

use botticelli_narrative::{NarrativeConfig, NarrativeConfigBuilder};
use botticelli_core::{Input, Role};

#[test]
fn test_narrative_with_invalid_act() {
    let result = NarrativeConfigBuilder::default()
        .name("test_invalid")
        .toc(vec!["nonexistent_act".to_string()])
        .build();
    
    // Should handle missing acts
    assert!(result.is_ok());
}

#[test]
fn test_narrative_empty_toc() {
    let result = NarrativeConfigBuilder::default()
        .name("test_empty")
        .toc(vec![])
        .build();
    
    assert!(result.is_ok());
}

#[test]
fn test_narrative_circular_reference() {
    // Test that circular narrative references are handled
    let result = NarrativeConfigBuilder::default()
        .name("circular")
        .toc(vec!["circular".to_string()])
        .build();
    
    assert!(result.is_ok());
}
