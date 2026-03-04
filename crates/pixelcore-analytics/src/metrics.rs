use prometheus::{Counter, Gauge, Histogram, HistogramOpts, Registry};
use std::sync::Arc;

/// Metrics collector
pub struct MetricsCollector {
    registry: Arc<Registry>,
    events_total: Counter,
    events_processing_duration: Histogram,
    warehouse_size: Gauge,
    etl_jobs_running: Gauge,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        let registry = Arc::new(Registry::new());

        let events_total = Counter::new("analytics_events_total", "Total number of events processed")
            .expect("Failed to create counter");
        registry.register(Box::new(events_total.clone()))
            .expect("Failed to register counter");

        let events_processing_duration = Histogram::with_opts(
            HistogramOpts::new("analytics_events_processing_duration_seconds", "Event processing duration")
        ).expect("Failed to create histogram");
        registry.register(Box::new(events_processing_duration.clone()))
            .expect("Failed to register histogram");

        let warehouse_size = Gauge::new("analytics_warehouse_size_bytes", "Warehouse size in bytes")
            .expect("Failed to create gauge");
        registry.register(Box::new(warehouse_size.clone()))
            .expect("Failed to register gauge");

        let etl_jobs_running = Gauge::new("analytics_etl_jobs_running", "Number of ETL jobs currently running")
            .expect("Failed to create gauge");
        registry.register(Box::new(etl_jobs_running.clone()))
            .expect("Failed to register gauge");

        Self {
            registry,
            events_total,
            events_processing_duration,
            warehouse_size,
            etl_jobs_running,
        }
    }

    /// Increment events counter
    pub fn inc_events(&self) {
        self.events_total.inc();
    }

    /// Record event processing duration
    pub fn record_processing_duration(&self, duration_secs: f64) {
        self.events_processing_duration.observe(duration_secs);
    }

    /// Set warehouse size
    pub fn set_warehouse_size(&self, size_bytes: f64) {
        self.warehouse_size.set(size_bytes);
    }

    /// Set ETL jobs running count
    pub fn set_etl_jobs_running(&self, count: f64) {
        self.etl_jobs_running.set(count);
    }

    /// Get registry
    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    /// Get metrics as text
    pub fn metrics_text(&self) -> String {
        use prometheus::Encoder;
        let encoder = prometheus::TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector() {
        let collector = MetricsCollector::new();

        collector.inc_events();
        collector.record_processing_duration(0.5);
        collector.set_warehouse_size(1024.0);
        collector.set_etl_jobs_running(2.0);

        let metrics = collector.metrics_text();
        assert!(metrics.contains("analytics_events_total"));
    }
}
