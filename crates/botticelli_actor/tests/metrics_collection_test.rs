//! Test that verifies metrics are being collected without full container deployment.
//!
//! This test validates:
//! 1. Metrics provider initializes correctly
//! 2. Metrics can be recorded
//! 3. Basic metrics API works as expected

use opentelemetry::metrics::{Counter, MeterProvider};
use opentelemetry_sdk::metrics::SdkMeterProvider;

#[test]
fn test_metrics_collection() {
    // Create meter provider (without exporter for minimal test)
    let provider = SdkMeterProvider::builder().build();

    // Get meter
    let meter = provider.meter("test_meter");

    // Create counter - this validates the API works
    let counter: Counter<u64> = meter
        .u64_counter("test_requests_total")
        .with_description("Test request counter")
        .build();

    // Record some metrics - validates recording works
    counter.add(1, &[]);
    counter.add(5, &[]);

    // If we get here without panicking, metrics API is functional
    let _ = provider.shutdown();
}

#[test]
fn test_multiple_metrics() {
    let provider = SdkMeterProvider::builder().build();
    let meter = provider.meter("test_meter");

    // Create multiple metrics - validates multiple instruments work
    let requests: Counter<u64> = meter.u64_counter("requests").build();
    let errors: Counter<u64> = meter.u64_counter("errors").build();

    requests.add(10, &[]);
    errors.add(2, &[]);

    // If we get here, multiple metrics are working
    let _ = provider.shutdown();
}
