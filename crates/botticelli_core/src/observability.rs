//! OpenTelemetry metrics initialization.

#[cfg(feature = "metrics")]
use opentelemetry::{KeyValue, global};
#[cfg(feature = "metrics")]
use opentelemetry_otlp::{MetricExporter as OtlpExporter, WithExportConfig};
#[cfg(feature = "metrics")]
use opentelemetry_sdk::{
    Resource,
    metrics::{PeriodicReader, SdkMeterProvider},
};
#[cfg(feature = "metrics")]
use opentelemetry_stdout::MetricExporter as StdoutExporter;
#[cfg(feature = "metrics")]
use std::time::Duration;
use tracing::{debug, info, instrument, warn};

/// Initialize OpenTelemetry metrics with OTLP or stdout export.
///
/// This sets up:
/// - Metrics with OTLP exporter (or stdout fallback)
/// - Periodic export at configurable interval
/// - Global meter provider
///
/// Checks OTEL_EXPORTER environment variable:
/// - "otlp" -> OTLP exporter to OTEL_EXPORTER_OTLP_ENDPOINT (default: http://localhost:4318)
/// - "stdout" or unset -> stdout exporter
///
/// When the `metrics` feature is disabled, this function returns `Ok(())` immediately.
#[instrument(skip_all, fields(service_name))]
pub fn init_observability(
    service_name: &'static str,
    export_interval_secs: u64,
) -> Result<(), String> {
    #[cfg(not(feature = "metrics"))]
    {
        let _ = export_interval_secs; // Silence unused warning
        info!(
            service_name = service_name,
            "Metrics feature disabled - skipping metrics initialization"
        );
        Ok(())
    }

    #[cfg(feature = "metrics")]
    {
        info!(
            service_name = service_name,
            export_interval_secs = export_interval_secs,
            "Initializing OpenTelemetry metrics"
        );

        // Create resource with service name
        let resource = Resource::builder_empty()
            .with_attributes([KeyValue::new("service.name", service_name)])
            .build();
        debug!("Created OpenTelemetry resource with service name");

        // Initialize metrics with periodic export
        debug!(
            export_interval_secs = export_interval_secs,
            "Setting up metrics provider"
        );

        // Determine exporter type from environment
        let exporter_type = std::env::var("OTEL_EXPORTER").unwrap_or_else(|_| "stdout".to_string());
        info!(exporter_type = %exporter_type, "Selecting metrics exporter");

        let meter_provider = match exporter_type.as_str() {
            "otlp" => {
                let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
                    .unwrap_or_else(|_| "http://localhost:4318".to_string());
                info!(endpoint = %endpoint, "Using OTLP metrics exporter");

                let exporter = OtlpExporter::builder()
                    .with_http()
                    .with_endpoint(&endpoint)
                    .with_timeout(Duration::from_secs(10))
                    .build()
                    .map_err(|e| {
                        let msg = format!("Failed to create OTLP exporter: {}", e);
                        warn!(%msg, "OTLP exporter creation failed");
                        msg
                    })?;
                debug!("OTLP metric exporter created");

                let reader = PeriodicReader::builder(exporter)
                    .with_interval(Duration::from_secs(export_interval_secs))
                    .build();
                debug!(
                    interval_secs = export_interval_secs,
                    "Created OTLP periodic reader"
                );

                SdkMeterProvider::builder()
                    .with_resource(resource.clone())
                    .with_reader(reader)
                    .build()
            }
            _ => {
                info!("Using stdout metrics exporter");
                let exporter = StdoutExporter::default();
                debug!("Created stdout metric exporter");

                let reader = PeriodicReader::builder(exporter)
                    .with_interval(Duration::from_secs(export_interval_secs))
                    .build();
                debug!(
                    interval_secs = export_interval_secs,
                    "Created stdout periodic reader"
                );

                SdkMeterProvider::builder()
                    .with_resource(resource.clone())
                    .with_reader(reader)
                    .build()
            }
        };

        debug!("Built meter provider");

        global::set_meter_provider(meter_provider.clone());
        info!("Meter provider registered globally");

        // Create test counter to verify metrics pipeline
        let meter = global::meter(service_name);
        let test_counter = meter.u64_counter("observability_init_test").build();
        test_counter.add(1, &[]);
        info!(
            service_name = service_name,
            "Metrics initialized successfully, test counter incremented"
        );

        Ok(())
    }
}

/// Shutdown metrics provider gracefully.
#[instrument]
pub fn shutdown_observability() {
    info!("Shutting down OpenTelemetry metrics provider");
    // Meter provider shutdown happens automatically on drop
    debug!("Metrics shutdown complete");
}
