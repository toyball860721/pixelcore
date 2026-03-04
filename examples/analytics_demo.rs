//! Data Warehouse and Analytics Demo
//!
//! This example demonstrates the data warehouse, ETL pipeline,
//! and analytics capabilities.

use pixelcore_analytics::{
    DataWarehouse, WarehouseConfig, EtlPipeline, EtlJob,
    etl::{EtlJobConfig, SourceType, TransformRule, TransformType},
    MetricsCollector,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("📊 PixelCore Data Warehouse & Analytics Demo\n");

    // Initialize metrics collector
    println!("Initializing metrics collector...");
    let metrics = MetricsCollector::new();
    println!("✅ Metrics collector initialized\n");

    // Initialize data warehouse
    println!("Initializing data warehouse...");
    let config = WarehouseConfig {
        host: std::env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".to_string()),
        port: std::env::var("POSTGRES_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(5432),
        dbname: "pixelcore_analytics".to_string(),
        user: std::env::var("POSTGRES_USER").unwrap_or_else(|_| "postgres".to_string()),
        password: std::env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "postgres".to_string()),
        pool_size: 10,
    };

    let warehouse = match DataWarehouse::new(config).await {
        Ok(w) => Arc::new(w),
        Err(e) => {
            eprintln!("⚠️  Warning: Could not connect to PostgreSQL: {}", e);
            eprintln!("   Make sure PostgreSQL is running:");
            eprintln!("   docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=postgres postgres\n");
            return Ok(());
        }
    };

    println!("✅ Data warehouse connected");

    // Initialize schema
    println!("Initializing warehouse schema...");
    warehouse.initialize().await?;
    println!("✅ Schema initialized\n");

    // Insert sample events
    println!("=== Inserting Sample Events ===\n");

    let events = vec![
        ("page_view", serde_json::json!({"page": "/home", "duration": 2.5})),
        ("page_view", serde_json::json!({"page": "/products", "duration": 5.2})),
        ("click", serde_json::json!({"element": "buy_button", "product_id": "123"})),
        ("purchase", serde_json::json!({"product_id": "123", "amount": 99.99})),
        ("page_view", serde_json::json!({"page": "/checkout", "duration": 3.1})),
    ];

    for (event_type, data) in events {
        let id = warehouse.insert_event(event_type, None, data).await?;
        println!("  Inserted event: {} ({})", event_type, id);
        metrics.inc_events();
    }

    println!();

    // Insert sample metrics
    println!("=== Inserting Sample Metrics ===\n");

    let sample_metrics = vec![
        ("response_time_ms", 45.2),
        ("cpu_usage_percent", 65.5),
        ("memory_usage_mb", 512.0),
        ("active_users", 150.0),
        ("requests_per_second", 1250.0),
    ];

    for (metric_name, value) in sample_metrics {
        let id = warehouse.insert_metric(
            metric_name,
            value,
            Some(serde_json::json!({"region": "us-east"})),
        ).await?;
        println!("  Inserted metric: {} = {} ({})", metric_name, value, id);
    }

    println!();

    // Query events
    println!("=== Querying Events ===\n");

    let events = warehouse.query_events(
        Some("page_view"),
        None,
        None,
        None,
        10,
    ).await?;

    println!("Found {} page_view events:", events.len());
    for event in events {
        println!("  - {} at {}: {:?}", event.event_type, event.timestamp, event.data);
    }

    println!();

    // Create ETL pipeline
    println!("=== ETL Pipeline Demo ===\n");

    let mut pipeline = EtlPipeline::new();

    // Add ETL job 1: Database extraction
    let job1_config = EtlJobConfig {
        name: "database_sync".to_string(),
        source_type: SourceType::Database {
            connection_string: "postgresql://localhost/source_db".to_string(),
        },
        transform_rules: vec![
            TransformRule {
                rule_type: TransformType::Filter,
                field: "status".to_string(),
                params: serde_json::json!({"value": "active"}),
            },
        ],
        schedule: Some("0 * * * *".to_string()), // Every hour
    };

    let job1 = EtlJob::new(job1_config, warehouse.clone());
    pipeline.add_job(job1);

    // Add ETL job 2: API extraction
    let job2_config = EtlJobConfig {
        name: "api_sync".to_string(),
        source_type: SourceType::Api {
            endpoint: "https://api.example.com/data".to_string(),
            auth_token: Some("token123".to_string()),
        },
        transform_rules: vec![
            TransformRule {
                rule_type: TransformType::Map,
                field: "timestamp".to_string(),
                params: serde_json::json!({"format": "iso8601"}),
            },
        ],
        schedule: Some("*/15 * * * *".to_string()), // Every 15 minutes
    };

    let job2 = EtlJob::new(job2_config, warehouse.clone());
    pipeline.add_job(job2);

    println!("Created ETL pipeline with 2 jobs");

    // Run pipeline
    println!("Running ETL pipeline...");
    metrics.set_etl_jobs_running(2.0);

    match pipeline.run_all().await {
        Ok(_) => {
            println!("✅ Pipeline completed successfully");

            // Get job statuses
            let statuses = pipeline.statuses().await;
            println!("\nJob Statuses:");
            for (i, status) in statuses.iter().enumerate() {
                println!("  Job {}: {:?}", i + 1, status.state);
                println!("    Records processed: {}", status.records_processed);
                println!("    Records failed: {}", status.records_failed);
            }
        }
        Err(e) => {
            eprintln!("❌ Pipeline failed: {}", e);
        }
    }

    metrics.set_etl_jobs_running(0.0);

    println!();

    // Display metrics
    println!("=== Metrics Summary ===\n");
    println!("{}", metrics.metrics_text());

    println!("🎉 Demo completed successfully!");

    Ok(())
}
