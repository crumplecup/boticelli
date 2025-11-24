//! Integration tests for the actor server binary and configuration system.

use botticelli_actor::ActorConfig;
use tempfile::TempDir;

#[test]
fn test_actor_config_loading() {
    let config_content = r#"
[actor]
name = "test_actor"
description = "Test actor description"
knowledge = ["test_content"]
skills = ["test_skill"]
"#;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("test_actor.toml");
    std::fs::write(&config_path, config_content).expect("Failed to write config file");

    let config = ActorConfig::from_file(&config_path).expect("Failed to load config");

    assert_eq!(config.name(), "test_actor");
    assert_eq!(config.description(), "Test actor description");
    assert_eq!(config.knowledge().len(), 1);
    assert_eq!(config.knowledge()[0], "test_content");
}

#[test]
fn test_actor_config_with_skills() {
    let config_content = r#"
[actor]
name = "skill_actor"
description = "Test actor with skills"
knowledge = []
skills = ["test_skill", "another_skill"]
"#;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("skill_actor.toml");
    std::fs::write(&config_path, config_content).expect("Failed to write config file");

    let config = ActorConfig::from_file(&config_path).expect("Failed to load config");

    assert_eq!(config.name(), "skill_actor");
    assert_eq!(config.skills().len(), 2);
    // skill_configs is optional and may be empty
}

#[test]
fn test_multiple_knowledge_sources() {
    let config_content = r#"
[actor]
name = "multi_knowledge_actor"
description = "Actor with multiple knowledge sources"
knowledge = ["content_table_1", "content_table_2"]
skills = []
"#;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("multi_knowledge.toml");
    std::fs::write(&config_path, config_content).expect("Failed to write config file");

    let config = ActorConfig::from_file(&config_path).expect("Failed to load config");

    assert_eq!(config.knowledge().len(), 2);
    assert_eq!(config.knowledge()[0], "content_table_1");
    assert_eq!(config.knowledge()[1], "content_table_2");
}

#[test]
fn test_actor_config_minimal() {
    let config_content = r#"
[actor]
name = "minimal_actor"
description = "Minimal test"
knowledge = []
skills = []
"#;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("minimal.toml");
    std::fs::write(&config_path, config_content).expect("Failed to write config file");

    let result = ActorConfig::from_file(&config_path);
    assert!(result.is_ok());
    
    let config = result.unwrap();
    assert_eq!(config.name(), "minimal_actor");
    assert_eq!(config.description(), "Minimal test");
}
