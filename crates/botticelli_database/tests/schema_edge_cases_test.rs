//! Tests for schema inference edge cases

use botticelli_database::{create_pool, infer_schema_from_json};
use serde_json::json;

#[test]
fn test_schema_inference_nested_objects() {
    let data = json!({
        "user": {
            "name": "Alice",
            "age": 30
        }
    });
    
    let result = infer_schema_from_json(&data);
    assert!(result.is_ok());
}

#[test]
fn test_schema_inference_arrays() {
    let data = json!({
        "tags": ["rust", "async", "testing"]
    });
    
    let result = infer_schema_from_json(&data);
    assert!(result.is_ok());
}

#[test]
fn test_schema_inference_null_values() {
    let data = json!({
        "optional_field": null,
        "required_field": "value"
    });
    
    let result = infer_schema_from_json(&data);
    assert!(result.is_ok());
}

#[test]
fn test_schema_inference_mixed_types() {
    let data = json!({
        "id": 123,
        "name": "test",
        "active": true,
        "score": 99.5
    });
    
    let result = infer_schema_from_json(&data);
    assert!(result.is_ok());
}
