use botticelli_core::{init_observability, shutdown_observability};

#[test]
fn test_init_observability_without_metrics() {
    // Test that observability initializes successfully with metrics disabled
    let result = init_observability();
    assert!(
        result.is_ok(),
        "Observability initialization should succeed without metrics feature: {:?}",
        result.err()
    );

    // Clean up
    shutdown_observability();
}

#[test]
#[cfg(feature = "metrics")]
fn test_init_observability_with_metrics() {
    // Test that observability initializes successfully with metrics enabled
    let result = init_observability();
    assert!(
        result.is_ok(),
        "Observability initialization should succeed with metrics feature: {:?}",
        result.err()
    );

    // Clean up
    shutdown_observability();
}
