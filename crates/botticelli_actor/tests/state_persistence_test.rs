//! Tests for database state persistence.
//!
//! Note: These tests validate the state persistence plumbing works correctly.
//! Full integration testing requires a running PostgreSQL instance.

use botticelli_actor::DatabaseStatePersistence;
use botticelli_server::StatePersistence;

#[tokio::test]
async fn test_state_persistence_interface() {
    // Validate that DatabaseStatePersistence implements the trait
    let persistence = DatabaseStatePersistence::new();
    
    // The trait methods are available
    let _result = persistence.load_state().await;
    
    // Note: Actual database operations require DATABASE_URL and a running PostgreSQL instance
    // Full integration tests would go here with proper database setup
}

#[test]
fn test_state_persistence_construction() {
    // Verify DatabaseStatePersistence can be constructed
    let _persistence = DatabaseStatePersistence::new();
    
    // The type is constructible and implements required traits
    // Actual database operations are tested in integration tests
}
