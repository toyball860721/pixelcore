//! Connection pool demo
//!
//! This example demonstrates how to use the connection pool
//! to reuse database connections efficiently.

use pixelcore_runtime::{ConnectionPool, PoolStats};
use std::time::Duration;

#[derive(Clone)]
struct DbConnection {
    id: String,
}

impl DbConnection {
    async fn new(id: String) -> Result<Self, String> {
        println!("Creating new connection: {}", id);
        // Simulate connection creation delay
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(Self { id })
    }

    async fn query(&self, sql: &str) -> Result<String, String> {
        println!("Executing query on {}: {}", self.id, sql);
        Ok(format!("Result from {}", self.id))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Connection Pool Demo ===\n");

    // Create a connection pool with max 3 connections and 60s idle timeout
    let pool = ConnectionPool::new(3, Duration::from_secs(60));

    // First request - creates new connection
    println!("1. First request:");
    let conn1 = pool
        .get_or_create("db1", || DbConnection::new("conn-1".to_string()))
        .await?;
    conn1.query("SELECT * FROM users").await?;

    let stats = pool.stats("db1").await;
    print_stats("After first request", &stats);

    // Release connection
    pool.release("db1", &conn1).await;

    let stats = pool.stats("db1").await;
    print_stats("After release", &stats);

    // Second request - reuses existing connection
    println!("\n2. Second request (should reuse connection):");
    let conn2 = pool
        .get_or_create("db1", || DbConnection::new("conn-2".to_string()))
        .await?;
    conn2.query("SELECT * FROM orders").await?;

    let stats = pool.stats("db1").await;
    print_stats("After second request", &stats);

    // Third request - creates another connection
    println!("\n3. Third request (conn2 still in use, creates new):");
    let conn3 = pool
        .get_or_create("db1", || DbConnection::new("conn-3".to_string()))
        .await?;
    conn3.query("SELECT * FROM products").await?;

    let stats = pool.stats("db1").await;
    print_stats("After third request", &stats);

    // Release both connections
    pool.release("db1", &conn2).await;
    pool.release("db1", &conn3).await;

    let stats = pool.stats("db1").await;
    print_stats("After releasing both", &stats);

    // Fourth request - reuses one of the idle connections
    println!("\n4. Fourth request (should reuse idle connection):");
    let conn4 = pool
        .get_or_create("db1", || DbConnection::new("conn-4".to_string()))
        .await?;
    conn4.query("SELECT * FROM inventory").await?;

    let stats = pool.stats("db1").await;
    print_stats("After fourth request", &stats);

    // Demonstrate multiple pools
    println!("\n5. Using different pool keys:");
    let conn_db2 = pool
        .get_or_create("db2", || DbConnection::new("db2-conn-1".to_string()))
        .await?;
    conn_db2.query("SELECT * FROM logs").await?;

    let stats_db1 = pool.stats("db1").await;
    let stats_db2 = pool.stats("db2").await;
    print_stats("db1 pool", &stats_db1);
    print_stats("db2 pool", &stats_db2);

    println!("\n=== Demo completed successfully ===");
    Ok(())
}

fn print_stats(label: &str, stats: &PoolStats) {
    println!(
        "  {} - Total: {}, In use: {}, Idle: {}",
        label, stats.total, stats.in_use, stats.idle
    );
}
