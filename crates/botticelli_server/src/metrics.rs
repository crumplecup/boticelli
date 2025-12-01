//! Metrics collection for Botticelli bot server.
//!
//! Provides OpenTelemetry-based metrics for bots, narratives, and the content pipeline.
//!
//! Available with the `metrics` feature.

#[cfg(feature = "metrics")]
use opentelemetry::{
    KeyValue, global,
    metrics::{Counter, Gauge, Histogram, Meter},
};
#[cfg(feature = "metrics")]
use std::sync::Arc;

/// Bot-level metrics for tracking execution and health.
///
/// Available with the `metrics` feature.
#[cfg(feature = "metrics")]
#[derive(Clone)]
pub struct BotMetrics {
    /// Meter handle kept alive for metric instruments
    _meter: Meter,
    /// Total bot executions
    pub executions: Counter<u64>,
    /// Total bot failures
    pub failures: Counter<u64>,
    /// Bot execution duration in seconds
    pub duration: Histogram<f64>,
    /// Pending content count in queue
    pub queue_depth: Gauge<u64>,
    /// Time since last successful execution in seconds
    pub time_since_success: Gauge<u64>,
}

#[cfg(feature = "metrics")]
impl BotMetrics {
    /// Create new bot metrics.
    pub fn new() -> Self {
        use tracing::debug;
        debug!("Getting global meter for botticelli_bots");
        let meter = global::meter("botticelli_bots");
        debug!("Building bot metrics instruments");

        let executions = meter
            .u64_counter("bot.executions")
            .with_description("Total bot executions")
            .build();
        debug!("Created bot.executions counter");

        let failures = meter
            .u64_counter("bot.failures")
            .with_description("Total bot failures")
            .build();
        debug!("Created bot.failures counter");

        let duration = meter
            .f64_histogram("bot.duration")
            .with_unit("seconds")
            .with_description("Bot execution duration")
            .build();
        debug!("Created bot.duration histogram");

        let queue_depth = meter
            .u64_gauge("bot.queue_depth")
            .with_description("Pending content count in queue")
            .build();
        debug!("Created bot.queue_depth gauge");

        let time_since_success = meter
            .u64_gauge("bot.time_since_success")
            .with_unit("seconds")
            .with_description("Time since last successful execution")
            .build();
        debug!("Created bot.time_since_success gauge");

        debug!("BotMetrics instruments created successfully");
        Self {
            _meter: meter.clone(),
            executions,
            failures,
            duration,
            queue_depth,
            time_since_success,
        }
    }

    /// Record a successful execution.
    pub fn record_execution(&self, bot_type: &str, duration_secs: f64) {
        use tracing::debug;
        let labels = &[KeyValue::new("bot_type", bot_type.to_string())];
        debug!(
            bot_type = bot_type,
            duration_secs = duration_secs,
            "Recording bot execution metrics"
        );
        self.executions.add(1, labels);
        self.duration.record(duration_secs, labels);
        self.time_since_success.record(0, labels);
        debug!("Bot execution metrics recorded");
    }

    /// Record a failed execution.
    pub fn record_failure(&self, bot_type: &str) {
        use tracing::debug;
        let labels = &[KeyValue::new("bot_type", bot_type.to_string())];
        debug!(bot_type = bot_type, "Recording bot failure metric");
        self.failures.add(1, labels);
        debug!("Bot failure metric recorded");
    }

    /// Update queue depth.
    pub fn update_queue_depth(&self, bot_type: &str, depth: u64) {
        let labels = &[KeyValue::new("bot_type", bot_type.to_string())];
        self.queue_depth.record(depth, labels);
    }
}

#[cfg(feature = "metrics")]
impl Default for BotMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Narrative-level metrics for tracking execution performance.
///
/// Available with the `metrics` feature.
#[cfg(feature = "metrics")]
#[derive(Clone)]
pub struct NarrativeMetrics {
    /// Meter handle kept alive for metric instruments
    _meter: Meter,
    /// Narrative execution count
    pub executions: Counter<u64>,
    /// Narrative execution duration
    pub duration: Histogram<f64>,
    /// Act execution duration
    pub act_duration: Histogram<f64>,
    /// JSON extraction success count
    pub json_success: Counter<u64>,
    /// JSON extraction failure count
    pub json_failures: Counter<u64>,
}

#[cfg(feature = "metrics")]
impl NarrativeMetrics {
    /// Create new narrative metrics.
    pub fn new() -> Self {
        let meter = global::meter("botticelli_narratives");

        Self {
            _meter: meter.clone(),
            executions: meter
                .u64_counter("narrative.executions")
                .with_description("Narrative execution count")
                .build(),
            duration: meter
                .f64_histogram("narrative.duration")
                .with_unit("seconds")
                .with_description("Narrative execution duration")
                .build(),
            act_duration: meter
                .f64_histogram("narrative.act.duration")
                .with_unit("seconds")
                .with_description("Act execution duration")
                .build(),
            json_success: meter
                .u64_counter("narrative.json.success")
                .with_description("JSON extraction successes")
                .build(),
            json_failures: meter
                .u64_counter("narrative.json.failures")
                .with_description("JSON extraction failures")
                .build(),
        }
    }

    /// Record narrative execution.
    pub fn record_execution(&self, narrative_name: &str, duration_secs: f64, success: bool) {
        let labels = &[
            KeyValue::new("narrative_name", narrative_name.to_string()),
            KeyValue::new("success", success),
        ];
        self.executions.add(1, labels);
        self.duration.record(duration_secs, labels);
    }

    /// Record act execution.
    pub fn record_act(&self, act_name: &str, duration_secs: f64) {
        let labels = &[KeyValue::new("act_name", act_name.to_string())];
        self.act_duration.record(duration_secs, labels);
    }

    /// Record JSON extraction result.
    pub fn record_json_extraction(&self, narrative_name: &str, success: bool) {
        let labels = &[KeyValue::new("narrative_name", narrative_name.to_string())];
        if success {
            self.json_success.add(1, labels);
        } else {
            self.json_failures.add(1, labels);
        }
    }
}

#[cfg(feature = "metrics")]
impl Default for NarrativeMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Content pipeline metrics.
///
/// Available with the `metrics` feature.
#[cfg(feature = "metrics")]
#[derive(Clone)]
pub struct PipelineMetrics {
    /// Meter handle kept alive for metric instruments
    _meter: Meter,
    /// Posts generated
    pub generated: Counter<u64>,
    /// Posts curated
    pub curated: Counter<u64>,
    /// Posts published
    pub published: Counter<u64>,
    /// Pipeline stage latency
    pub stage_latency: Histogram<f64>,
}

#[cfg(feature = "metrics")]
impl PipelineMetrics {
    /// Create new pipeline metrics.
    pub fn new() -> Self {
        let meter = global::meter("botticelli_pipeline");

        Self {
            _meter: meter.clone(),
            generated: meter
                .u64_counter("pipeline.generated")
                .with_description("Posts generated")
                .build(),
            curated: meter
                .u64_counter("pipeline.curated")
                .with_description("Posts curated")
                .build(),
            published: meter
                .u64_counter("pipeline.published")
                .with_description("Posts published")
                .build(),
            stage_latency: meter
                .f64_histogram("pipeline.stage.latency")
                .with_unit("seconds")
                .with_description("Pipeline stage latency")
                .build(),
        }
    }

    /// Record content generation.
    pub fn record_generated(&self, count: u64) {
        self.generated.add(count, &[]);
    }

    /// Record content curation.
    pub fn record_curated(&self, count: u64) {
        self.curated.add(count, &[]);
    }

    /// Record content publication.
    pub fn record_published(&self, count: u64) {
        self.published.add(count, &[]);
    }

    /// Record stage latency.
    pub fn record_stage_latency(&self, stage: &str, latency_secs: f64) {
        let labels = &[KeyValue::new("stage", stage.to_string())];
        self.stage_latency.record(latency_secs, labels);
    }
}

#[cfg(feature = "metrics")]
impl Default for PipelineMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Aggregated metrics for the entire bot server.
///
/// Available with the `metrics` feature.
#[cfg(feature = "metrics")]
#[derive(Clone)]
pub struct ServerMetrics {
    /// Bot-level metrics
    pub bots: Arc<BotMetrics>,
    /// Narrative-level metrics
    pub narratives: Arc<NarrativeMetrics>,
    /// Pipeline-level metrics
    pub pipeline: Arc<PipelineMetrics>,
}

#[cfg(feature = "metrics")]
impl ServerMetrics {
    /// Create new server metrics.
    pub fn new() -> Self {
        use tracing::{debug, info};
        info!("Initializing ServerMetrics");
        debug!("Creating BotMetrics");
        let bots = Arc::new(BotMetrics::new());
        debug!("Creating NarrativeMetrics");
        let narratives = Arc::new(NarrativeMetrics::new());
        debug!("Creating PipelineMetrics");
        let pipeline = Arc::new(PipelineMetrics::new());
        info!("ServerMetrics initialized successfully");
        Self {
            bots,
            narratives,
            pipeline,
        }
    }
}

#[cfg(feature = "metrics")]
impl Default for ServerMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Metrics collector for HTTP API snapshots.
///
/// Provides a simplified view of metrics for JSON export via HTTP API.
/// Uses Arc internally so cloning is cheap.
///
/// Available with the `metrics` feature.
#[cfg(feature = "metrics")]
#[derive(Clone)]
pub struct MetricsCollector {
    /// Server metrics handle kept alive
    _metrics: Arc<ServerMetrics>,
}

#[cfg(feature = "metrics")]
impl MetricsCollector {
    /// Create new metrics collector.
    pub fn new() -> Self {
        Self {
            _metrics: Arc::new(ServerMetrics::new()),
        }
    }

    /// Create from existing server metrics.
    pub fn from_server_metrics(metrics: Arc<ServerMetrics>) -> Self {
        Self { _metrics: metrics }
    }

    /// Get current metrics snapshot.
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            bots: BotMetricsSnapshot::default(),
            narratives: NarrativeMetricsSnapshot::default(),
            pipeline: PipelineMetricsSnapshot::default(),
        }
    }
}

#[cfg(feature = "metrics")]
impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of metrics at a point in time.
/// Metrics snapshot for serialization.
///
/// Available with the `metrics` feature.
#[cfg(feature = "metrics")]
#[derive(Debug, Clone, Default, serde::Serialize, derive_getters::Getters)]
pub struct MetricsSnapshot {
    /// Bot metrics snapshot
    bots: BotMetricsSnapshot,
    /// Narrative metrics snapshot
    narratives: NarrativeMetricsSnapshot,
    /// Pipeline metrics snapshot
    pipeline: PipelineMetricsSnapshot,
}

/// Bot metrics snapshot.
#[cfg(feature = "metrics")]
#[derive(Debug, Clone, Default, serde::Serialize, derive_getters::Getters)]
pub struct BotMetricsSnapshot {
    /// Total executions
    executions: u64,
    /// Total failures
    failures: u64,
    /// Average duration in seconds
    avg_duration: f64,
    /// Current queue depth
    queue_depth: u64,
}

/// Narrative metrics snapshot.
#[cfg(feature = "metrics")]
#[derive(Debug, Clone, Default, serde::Serialize, derive_getters::Getters)]
pub struct NarrativeMetricsSnapshot {
    /// Total executions
    executions: u64,
    /// Average duration in seconds
    avg_duration: f64,
    /// JSON success count
    json_success: u64,
    /// JSON failure count
    json_failures: u64,
    /// JSON success rate
    json_success_rate: f64,
}

/// Pipeline metrics snapshot.
#[cfg(feature = "metrics")]
#[derive(Debug, Clone, Default, serde::Serialize, derive_getters::Getters)]
pub struct PipelineMetricsSnapshot {
    /// Posts generated
    generated: u64,
    /// Posts curated
    curated: u64,
    /// Posts published
    published: u64,
    /// Average stage latency in seconds
    avg_stage_latency: f64,
}
