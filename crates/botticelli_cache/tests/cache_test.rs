//! Tests for the CommandCache implementation.

use botticelli_cache::{CommandCache, CommandCacheConfig};
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;

#[test]
fn test_cache_insert_and_get() {
    let config = CommandCacheConfig::default()
        .with_default_ttl(10)
        .with_max_size(100);
    let mut cache = CommandCache::new(config);

    let mut args = HashMap::new();
    args.insert("param1".to_string(), json!("value1"));

    cache.insert("discord", "test.command", &args, json!("result"), Some(10));

    let entry = cache.get("discord", "test.command", &args);
    assert!(entry.is_some());
    assert_eq!(entry.unwrap().value(), &json!("result"));

    // Non-existent command should return None
    assert!(cache.get("discord", "other.command", &args).is_none());
}

#[test]
fn test_cache_expiration() {
    let config = CommandCacheConfig::default().with_default_ttl(1); // 1 second TTL
    let mut cache = CommandCache::new(config);

    let mut args = HashMap::new();
    args.insert("param1".to_string(), json!("value1"));

    cache.insert("discord", "test.command", &args, json!("result"), Some(1));
    assert!(cache.get("discord", "test.command", &args).is_some());

    // Wait for expiration
    std::thread::sleep(Duration::from_secs(2));

    // Should be expired now
    assert!(cache.get("discord", "test.command", &args).is_none());
}

#[test]
fn test_cache_clear() {
    let config = CommandCacheConfig::default();
    let mut cache = CommandCache::new(config);

    let mut args1 = HashMap::new();
    args1.insert("param1".to_string(), json!("value1"));
    let mut args2 = HashMap::new();
    args2.insert("param2".to_string(), json!("value2"));

    cache.insert("discord", "cmd1", &args1, json!("result1"), None);
    cache.insert("discord", "cmd2", &args2, json!("result2"), None);

    assert_eq!(cache.len(), 2);

    cache.clear();

    assert_eq!(cache.len(), 0);
    assert!(cache.get("discord", "cmd1", &args1).is_none());
    assert!(cache.get("discord", "cmd2", &args2).is_none());
}

#[test]
fn test_cache_len() {
    let config = CommandCacheConfig::default();
    let mut cache = CommandCache::new(config);

    assert_eq!(cache.len(), 0);

    let mut args1 = HashMap::new();
    args1.insert("param1".to_string(), json!("value1"));

    cache.insert("discord", "cmd1", &args1, json!("result1"), None);
    assert_eq!(cache.len(), 1);

    let mut args2 = HashMap::new();
    args2.insert("param2".to_string(), json!("value2"));

    cache.insert("discord", "cmd2", &args2, json!("result2"), None);
    assert_eq!(cache.len(), 2);
}

#[test]
fn test_cache_is_empty() {
    let config = CommandCacheConfig::default();
    let mut cache = CommandCache::new(config);

    assert!(cache.is_empty());

    let mut args = HashMap::new();
    args.insert("param1".to_string(), json!("value1"));

    cache.insert("discord", "cmd1", &args, json!("result1"), None);
    assert!(!cache.is_empty());

    cache.clear();
    assert!(cache.is_empty());
}

#[test]
fn test_cache_update_existing_key() {
    let config = CommandCacheConfig::default();
    let mut cache = CommandCache::new(config);

    let mut args = HashMap::new();
    args.insert("param1".to_string(), json!("value1"));

    cache.insert("discord", "cmd1", &args, json!("result1"), None);
    let entry = cache.get("discord", "cmd1", &args);
    assert_eq!(entry.unwrap().value(), &json!("result1"));

    // Update with new value
    cache.insert("discord", "cmd1", &args, json!("result2"), None);
    let entry = cache.get("discord", "cmd1", &args);
    assert_eq!(entry.unwrap().value(), &json!("result2"));
}

#[test]
fn test_cache_cleanup_expired_entries() {
    let config = CommandCacheConfig::default().with_default_ttl(1);
    let mut cache = CommandCache::new(config);

    let mut args1 = HashMap::new();
    args1.insert("param1".to_string(), json!("value1"));
    let mut args2 = HashMap::new();
    args2.insert("param2".to_string(), json!("value2"));

    cache.insert("discord", "cmd1", &args1, json!("result1"), Some(1));
    cache.insert("discord", "cmd2", &args2, json!("result2"), Some(1));

    assert_eq!(cache.len(), 2);

    // Wait for expiration
    std::thread::sleep(Duration::from_secs(2));

    // Cleanup expired entries
    let removed = cache.cleanup_expired();
    assert_eq!(removed, 2);
    assert_eq!(cache.len(), 0);
}

#[test]
fn test_cache_lru_eviction() {
    let config = CommandCacheConfig::default().with_max_size(2);
    let mut cache = CommandCache::new(config);

    let mut args1 = HashMap::new();
    args1.insert("param1".to_string(), json!("value1"));
    let mut args2 = HashMap::new();
    args2.insert("param2".to_string(), json!("value2"));
    let mut args3 = HashMap::new();
    args3.insert("param3".to_string(), json!("value3"));

    cache.insert("discord", "cmd1", &args1, json!("result1"), None);
    cache.insert("discord", "cmd2", &args2, json!("result2"), None);

    assert_eq!(cache.len(), 2);

    // This should evict the least recently used entry (cmd1)
    cache.insert("discord", "cmd3", &args3, json!("result3"), None);

    assert_eq!(cache.len(), 2);
    assert!(cache.get("discord", "cmd1", &args1).is_none());
    assert!(cache.get("discord", "cmd2", &args2).is_some());
    assert!(cache.get("discord", "cmd3", &args3).is_some());
}
