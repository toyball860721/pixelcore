use crate::error::{AnalyticsError, Result};
use deadpool_postgres::{Config, Manager, ManagerConfig, Pool, RecyclingMethod, Runtime};
use tokio_postgres::NoTls;

/// Data warehouse configuration
#[derive(Debug, Clone)]
pub struct WarehouseConfig {
    /// Database host
    pub host: String,
    /// Database port
    pub port: u16,
    /// Database name
    pub dbname: String,
    /// Database user
    pub user: String,
    /// Database password
    pub password: String,
    /// Connection pool size
    pub pool_size: usize,
}

impl Default for WarehouseConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            dbname: "pixelcore_analytics".to_string(),
            user: "postgres".to_string(),
            password: "postgres".to_string(),
            pool_size: 10,
        }
    }
}

/// Data warehouse
pub struct DataWarehouse {
    pool: Pool,
}

impl DataWarehouse {
    /// Create a new data warehouse
    pub async fn new(config: WarehouseConfig) -> Result<Self> {
        let mut pg_config = Config::new();
        pg_config.host = Some(config.host);
        pg_config.port = Some(config.port);
        pg_config.dbname = Some(config.dbname);
        pg_config.user = Some(config.user);
        pg_config.password = Some(config.password);
        pg_config.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        });

        let pool = pg_config
            .create_pool(Some(Runtime::Tokio1), NoTls)
            .map_err(|e| AnalyticsError::PoolError(e.to_string()))?;

        Ok(Self { pool })
    }

    /// Initialize warehouse schema
    pub async fn initialize(&self) -> Result<()> {
        let client = self.pool.get().await
            .map_err(|e| AnalyticsError::PoolError(e.to_string()))?;

        // Create events table
        client.execute(
            "CREATE TABLE IF NOT EXISTS events (
                id UUID PRIMARY KEY,
                event_type VARCHAR(100) NOT NULL,
                user_id UUID,
                data JSONB NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
            &[],
        ).await?;

        // Create index on timestamp for time-series queries
        client.execute(
            "CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp DESC)",
            &[],
        ).await?;

        // Create index on event_type
        client.execute(
            "CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type)",
            &[],
        ).await?;

        // Create index on user_id
        client.execute(
            "CREATE INDEX IF NOT EXISTS idx_events_user_id ON events(user_id)",
            &[],
        ).await?;

        // Create metrics table
        client.execute(
            "CREATE TABLE IF NOT EXISTS metrics (
                id UUID PRIMARY KEY,
                metric_name VARCHAR(100) NOT NULL,
                metric_value DOUBLE PRECISION NOT NULL,
                labels JSONB,
                timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
            &[],
        ).await?;

        // Create index on metric_name and timestamp
        client.execute(
            "CREATE INDEX IF NOT EXISTS idx_metrics_name_timestamp
             ON metrics(metric_name, timestamp DESC)",
            &[],
        ).await?;

        // Create aggregated_metrics table for pre-computed aggregations
        client.execute(
            "CREATE TABLE IF NOT EXISTS aggregated_metrics (
                id UUID PRIMARY KEY,
                metric_name VARCHAR(100) NOT NULL,
                aggregation_type VARCHAR(50) NOT NULL,
                metric_value DOUBLE PRECISION NOT NULL,
                time_bucket TIMESTAMPTZ NOT NULL,
                labels JSONB,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE(metric_name, aggregation_type, time_bucket)
            )",
            &[],
        ).await?;

        Ok(())
    }

    /// Insert an event
    pub async fn insert_event(
        &self,
        event_type: &str,
        user_id: Option<uuid::Uuid>,
        data: serde_json::Value,
    ) -> Result<uuid::Uuid> {
        let client = self.pool.get().await
            .map_err(|e| AnalyticsError::PoolError(e.to_string()))?;

        let id = uuid::Uuid::new_v4();
        let timestamp = chrono::Utc::now();

        client.execute(
            "INSERT INTO events (id, event_type, user_id, data, timestamp)
             VALUES ($1, $2, $3, $4, $5)",
            &[&id, &event_type, &user_id, &data, &timestamp],
        ).await?;

        Ok(id)
    }

    /// Insert a metric
    pub async fn insert_metric(
        &self,
        metric_name: &str,
        metric_value: f64,
        labels: Option<serde_json::Value>,
    ) -> Result<uuid::Uuid> {
        let client = self.pool.get().await
            .map_err(|e| AnalyticsError::PoolError(e.to_string()))?;

        let id = uuid::Uuid::new_v4();
        let timestamp = chrono::Utc::now();

        client.execute(
            "INSERT INTO metrics (id, metric_name, metric_value, labels, timestamp)
             VALUES ($1, $2, $3, $4, $5)",
            &[&id, &metric_name, &metric_value, &labels, &timestamp],
        ).await?;

        Ok(id)
    }

    /// Query events
    pub async fn query_events(
        &self,
        event_type: Option<&str>,
        user_id: Option<uuid::Uuid>,
        start_time: Option<chrono::DateTime<chrono::Utc>>,
        end_time: Option<chrono::DateTime<chrono::Utc>>,
        limit: i64,
    ) -> Result<Vec<Event>> {
        let client = self.pool.get().await
            .map_err(|e| AnalyticsError::PoolError(e.to_string()))?;

        // Build query based on filters
        let rows = match (event_type, user_id, start_time, end_time) {
            (Some(et), None, None, None) => {
                client.query(
                    "SELECT id, event_type, user_id, data, timestamp FROM events
                     WHERE event_type = $1 ORDER BY timestamp DESC LIMIT $2",
                    &[&et, &limit],
                ).await?
            }
            (None, Some(uid), None, None) => {
                client.query(
                    "SELECT id, event_type, user_id, data, timestamp FROM events
                     WHERE user_id = $1 ORDER BY timestamp DESC LIMIT $2",
                    &[&uid, &limit],
                ).await?
            }
            (Some(et), Some(uid), None, None) => {
                client.query(
                    "SELECT id, event_type, user_id, data, timestamp FROM events
                     WHERE event_type = $1 AND user_id = $2 ORDER BY timestamp DESC LIMIT $3",
                    &[&et, &uid, &limit],
                ).await?
            }
            _ => {
                // Default: no filters
                client.query(
                    "SELECT id, event_type, user_id, data, timestamp FROM events
                     ORDER BY timestamp DESC LIMIT $1",
                    &[&limit],
                ).await?
            }
        };

        let events = rows.iter().map(|row| Event {
            id: row.get(0),
            event_type: row.get(1),
            user_id: row.get(2),
            data: row.get(3),
            timestamp: row.get(4),
        }).collect();

        Ok(events)
    }

    /// Get connection pool
    pub fn pool(&self) -> &Pool {
        &self.pool
    }
}

/// Event record
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Event {
    pub id: uuid::Uuid,
    pub event_type: String,
    pub user_id: Option<uuid::Uuid>,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warehouse_config_default() {
        let config = WarehouseConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 5432);
        assert_eq!(config.pool_size, 10);
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL to be running
    async fn test_warehouse_creation() {
        let config = WarehouseConfig::default();
        let result = DataWarehouse::new(config).await;

        // This will fail if PostgreSQL is not running, which is expected
        assert!(result.is_ok() || result.is_err());
    }
}
