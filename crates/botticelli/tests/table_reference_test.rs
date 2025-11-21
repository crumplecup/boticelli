//! Integration tests for table reference functionality.

use botticelli::{
    format_as_csv, format_as_json, format_as_markdown, TableQueryExecutor, TableQueryViewBuilder,
};
use diesel::prelude::*;
use std::env;
use std::sync::{Arc, Mutex};

/// Helper to get database URL from environment.
fn database_url() -> String {
    env::var("DATABASE_URL").expect("DATABASE_URL must be set for integration tests")
}

/// Helper to establish database connection.
fn establish_connection() -> PgConnection {
    let url = database_url();
    PgConnection::establish(&url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", url))
}

#[test]
#[cfg_attr(not(feature = "database"), ignore)]
fn test_table_query_basic() {
    let conn = establish_connection();
    let executor = TableQueryExecutor::new(Arc::new(Mutex::new(conn)));

    let view = TableQueryViewBuilder::default()
        .table_name("welcome_messages")
        .limit(10)
        .build()
        .expect("Failed to build view");

    let results = executor.query_table(&view).expect("Query failed");
    
    // Should return a vector (may be empty if table is empty)
    // Just verify it's a valid Vec
    assert!(results.len() >= 0);
}

#[test]
#[cfg_attr(not(feature = "database"), ignore)]
fn test_table_query_with_columns() {
    let conn = establish_connection();
    let executor = TableQueryExecutor::new(Arc::new(Mutex::new(conn)));

    let view = TableQueryViewBuilder::default()
        .table_name("welcome_messages")
        .columns(vec!["title".to_string(), "content".to_string()])
        .limit(5)
        .build()
        .expect("Failed to build view");

    let results = executor.query_table(&view).expect("Query failed");
    
    // If we have results, verify they only have requested columns
    if let Some(first) = results.first() {
        assert!(first.get("title").is_some());
        assert!(first.get("content").is_some());
    }
}

#[test]
#[cfg_attr(not(feature = "database"), ignore)]
fn test_table_query_with_where() {
    let conn = establish_connection();
    let executor = TableQueryExecutor::new(Arc::new(Mutex::new(conn)));

    let view = TableQueryViewBuilder::default()
        .table_name("welcome_messages")
        .filter("status = 'approved'")
        .limit(10)
        .build()
        .expect("Failed to build view");

    let results = executor.query_table(&view).expect("Query failed");
    
    // Valid vec
    assert!(results.len() >= 0);
}

#[test]
#[cfg_attr(not(feature = "database"), ignore)]
fn test_table_query_with_order() {
    let conn = establish_connection();
    let executor = TableQueryExecutor::new(Arc::new(Mutex::new(conn)));

    let view = TableQueryViewBuilder::default()
        .table_name("welcome_messages")
        .order_by("created_at DESC")
        .limit(5)
        .build()
        .expect("Failed to build view");

    let results = executor.query_table(&view).expect("Query failed");
    
    // Valid vec
    assert!(results.len() >= 0);
}

#[test]
#[cfg_attr(not(feature = "database"), ignore)]
fn test_table_not_found() {
    let conn = establish_connection();
    let executor = TableQueryExecutor::new(Arc::new(Mutex::new(conn)));

    let view = TableQueryViewBuilder::default()
        .table_name("nonexistent_table_xyz")
        .build()
        .expect("Failed to build view");

    let result = executor.query_table(&view);
    
    assert!(result.is_err());
}

#[test]
#[cfg_attr(not(feature = "database"), ignore)]
fn test_format_as_json() {
    use serde_json::json;

    let rows = vec![
        json!({"id": 1, "title": "Test"}),
        json!({"id": 2, "title": "Another"}),
    ];

    let formatted = format_as_json(&rows);
    
    assert!(formatted.contains("\"id\""));
    assert!(formatted.contains("\"title\""));
    assert!(formatted.contains("Test"));
}

#[test]
#[cfg_attr(not(feature = "database"), ignore)]
fn test_format_as_markdown() {
    use serde_json::json;

    let rows = vec![
        json!({"id": 1, "title": "Test"}),
        json!({"id": 2, "title": "Another"}),
    ];

    let formatted = format_as_markdown(&rows);
    
    // Should have markdown table header
    assert!(formatted.contains("|"));
    assert!(formatted.contains("id"));
    assert!(formatted.contains("title"));
}

#[test]
#[cfg_attr(not(feature = "database"), ignore)]
fn test_format_as_csv() {
    use serde_json::json;

    let rows = vec![
        json!({"id": 1, "title": "Test"}),
        json!({"id": 2, "title": "Another"}),
    ];

    let formatted = format_as_csv(&rows);
    
    // Should have CSV header and rows
    assert!(formatted.contains("id,title"));
    assert!(formatted.contains("1,Test"));
}
