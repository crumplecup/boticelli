//! Integration tests for actor configuration.

use botticelli_actor::{ActorConfigBuilder, ExecutionConfigBuilder};

#[test]
fn test_actor_config_minimal() {
    // Test that ActorConfig can be built with required fields
    let config = ActorConfigBuilder::default()
        .name("test_actor".to_string())
        .description("Test description".to_string())
        .knowledge(vec![])
        .skills(vec![])
        .build()
        .expect("Valid actor config");

    assert_eq!(config.name(), "test_actor");
}

#[test]
fn test_actor_config_with_description() {
    // Test that ActorConfig accepts description
    let config = ActorConfigBuilder::default()
        .name("test_actor".to_string())
        .description("Test actor configuration".to_string())
        .knowledge(vec![])
        .skills(vec![])
        .build()
        .expect("Valid actor config");

    assert_eq!(config.name(), "test_actor");
    assert_eq!(config.description(), "Test actor configuration");
}

#[test]
fn test_execution_config_defaults() {
    // Test that ExecutionConfig has defaults
    let config = ExecutionConfigBuilder::default()
        .build()
        .expect("Valid execution config");

    // Default is to stop on unrecoverable
    assert!(config.stop_on_unrecoverable());
}

#[test]
fn test_execution_config_stop_on_unrecoverable() {
    // Test that execution can be configured to stop on unrecoverable errors
    let config = ExecutionConfigBuilder::default()
        .stop_on_unrecoverable(true)
        .build()
        .expect("Valid execution config");

    assert!(config.stop_on_unrecoverable());
}

#[test]
fn test_actor_config_with_custom_execution() {
    // Test that actor can be configured with custom execution behavior
    let exec_config = ExecutionConfigBuilder::default()
        .stop_on_unrecoverable(false)
        .build()
        .expect("Valid execution config");

    let config = ActorConfigBuilder::default()
        .name("custom_execution_actor".to_string())
        .description("Test".to_string())
        .knowledge(vec![])
        .skills(vec![])
        .execution(exec_config)
        .build()
        .expect("Valid actor config");

    assert_eq!(config.name(), "custom_execution_actor");
    assert!(!config.execution().stop_on_unrecoverable());
}
