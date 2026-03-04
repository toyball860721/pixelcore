use serde::{Deserialize, Serialize};

/// Analytics query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsQuery {
    /// Metric name
    pub metric: String,
    /// Aggregation type
    pub aggregation: AggregationType,
    /// Time range
    pub time_range: TimeRange,
    /// Group by fields
    pub group_by: Option<Vec<String>>,
    /// Filters
    pub filters: Option<Vec<Filter>>,
}

/// Aggregation type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationType {
    Count,
    Sum,
    Average,
    Min,
    Max,
    Percentile(f64),
}

/// Time range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: chrono::DateTime<chrono::Utc>,
    pub end: chrono::DateTime<chrono::Utc>,
    pub interval: Option<TimeInterval>,
}

/// Time interval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeInterval {
    Minute,
    Hour,
    Day,
    Week,
    Month,
}

/// Filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    pub field: String,
    pub operator: FilterOperator,
    pub value: serde_json::Value,
}

/// Filter operator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    In,
    NotIn,
}

/// Query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub data: Vec<DataPoint>,
    pub total: usize,
    pub query_time_ms: u64,
}

/// Data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub value: f64,
    pub labels: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_query() {
        let query = AnalyticsQuery {
            metric: "page_views".to_string(),
            aggregation: AggregationType::Count,
            time_range: TimeRange {
                start: chrono::Utc::now() - chrono::Duration::hours(24),
                end: chrono::Utc::now(),
                interval: Some(TimeInterval::Hour),
            },
            group_by: None,
            filters: None,
        };

        assert_eq!(query.metric, "page_views");
    }

    #[test]
    fn test_data_point() {
        let point = DataPoint {
            timestamp: chrono::Utc::now(),
            value: 100.0,
            labels: Some(serde_json::json!({"region": "us-east"})),
        };

        assert_eq!(point.value, 100.0);
    }
}
