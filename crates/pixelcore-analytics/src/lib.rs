//! PixelCore Analytics Module
//!
//! Provides data warehouse, ETL pipeline, and analytics capabilities.

pub mod warehouse;
pub mod etl;
pub mod metrics;
pub mod query;
pub mod error;

pub use warehouse::{DataWarehouse, WarehouseConfig};
pub use etl::{EtlPipeline, EtlJob};
pub use metrics::MetricsCollector;
pub use query::{AnalyticsQuery, QueryResult};
pub use error::{AnalyticsError, Result};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Verify that all public exports are accessible
        let _config: Option<WarehouseConfig> = None;
        let _query: Option<AnalyticsQuery> = None;
    }
}
