//! Tests for server configuration parsing and validation.

use botticelli_actor::{ActorServerConfig, ScheduleConfig};

#[test]
fn test_parse_server_config() {
    let toml = r#"
[server]
check_interval_seconds = 30

[server.circuit_breaker]
max_consecutive_failures = 3

[[actors]]
name = "test_actor"
config_file = "actors/test.toml"
channel_id = "123456"
enabled = true

[actors.schedule]
type = "Interval"
seconds = 1800
"#;

    let config: ActorServerConfig = toml::from_str(toml).expect("Valid TOML");
    assert_eq!(config.server.check_interval_seconds, 30);
    assert_eq!(config.server.circuit_breaker.max_consecutive_failures, 3);
    assert_eq!(config.actors.len(), 1);
    assert_eq!(config.actors[0].name, "test_actor");
    assert_eq!(config.actors[0].config_file, "actors/test.toml");
    assert_eq!(config.actors[0].channel_id, Some("123456".to_string()));

    match &config.actors[0].schedule {
        ScheduleConfig::Interval { seconds } => assert_eq!(*seconds, 1800),
        _ => panic!("Expected Interval schedule"),
    }
}

#[test]
fn test_default_values() {
    let toml = r#"
[[actors]]
name = "minimal"
config_file = "test.toml"
"#;

    let config: ActorServerConfig = toml::from_str(toml).expect("Valid TOML");
    assert_eq!(config.server.check_interval_seconds, 60);
    assert_eq!(config.server.circuit_breaker.max_consecutive_failures, 5);
    assert!(config.actors[0].enabled);
}

#[test]
fn test_immediate_schedule() {
    let toml = r#"
[[actors]]
name = "immediate"
config_file = "test.toml"

[actors.schedule]
type = "Immediate"
"#;

    let config: ActorServerConfig = toml::from_str(toml).expect("Valid TOML");
    matches!(config.actors[0].schedule, ScheduleConfig::Immediate);
}
