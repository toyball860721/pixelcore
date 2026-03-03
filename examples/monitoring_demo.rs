use pixelcore_monitoring::{
    MetricsCollector, AlertManager, NotificationManager,
    AlertRule, NotificationChannel, AlertCondition, AlertSeverity, MetricType, Metric,
};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PixelCore Monitoring Demo ===\n");

    // 1. Create metrics collector
    println!("1. Creating metrics collector...");
    let collector = MetricsCollector::new();

    // Collect system metrics
    let sys_metrics = collector.collect_system_metrics();
    println!("Collected system metrics:");
    println!("  - CPU Usage: {:.2}%", sys_metrics.cpu_usage_percent);
    println!("  - Memory Usage: {:.2}% ({} / {} bytes)",
        sys_metrics.memory_usage_percent,
        sys_metrics.memory_used_bytes,
        sys_metrics.memory_total_bytes
    );
    println!("  - Disk Usage: {:.2}% ({} / {} bytes)",
        sys_metrics.disk_usage_percent,
        sys_metrics.disk_used_bytes,
        sys_metrics.disk_total_bytes
    );
    println!("  - Network: RX {} bytes, TX {} bytes",
        sys_metrics.network_rx_bytes,
        sys_metrics.network_tx_bytes
    );
    println!();

    // 2. Register custom metrics
    println!("2. Registering custom metrics...");
    let cpu_metric = Metric::new(
        "cpu_usage".to_string(),
        MetricType::Gauge,
        "CPU usage percentage".to_string(),
    ).with_unit("%".to_string());
    collector.register_metric(cpu_metric);

    let memory_metric = Metric::new(
        "memory_usage".to_string(),
        MetricType::Gauge,
        "Memory usage percentage".to_string(),
    ).with_unit("%".to_string());
    collector.register_metric(memory_metric);

    println!("Registered 2 custom metrics");
    println!();

    // 3. Create alert manager and add rules
    println!("3. Setting up alert rules...");
    let alert_manager = AlertManager::new();

    let cpu_rule = AlertRule::new(
        "High CPU Usage".to_string(),
        "CPU usage is above 80%".to_string(),
        "cpu_usage".to_string(),
        AlertCondition::GreaterThan,
        80.0,
        AlertSeverity::Warning,
    );
    alert_manager.add_rule(cpu_rule)?;

    let memory_rule = AlertRule::new(
        "High Memory Usage".to_string(),
        "Memory usage is above 90%".to_string(),
        "memory_usage".to_string(),
        AlertCondition::GreaterThan,
        90.0,
        AlertSeverity::Critical,
    );
    alert_manager.add_rule(memory_rule)?;

    println!("Added {} alert rules", alert_manager.get_rules().len());
    println!();

    // 4. Create notification manager and add channels
    println!("4. Setting up notification channels...");
    let notification_manager = NotificationManager::new();

    notification_manager.add_channel(NotificationChannel::Console)?;

    notification_manager.add_channel(NotificationChannel::Email {
        recipients: vec![
            "ops@example.com".to_string(),
            "admin@example.com".to_string(),
        ],
    })?;

    notification_manager.add_channel(NotificationChannel::Slack {
        webhook_url: "https://hooks.slack.com/services/T00000000/B00000000/XXXXXXXXXXXX".to_string(),
        channel: "alerts".to_string(),
    })?;

    notification_manager.add_channel(NotificationChannel::Webhook {
        url: "https://monitoring.example.com/alerts".to_string(),
    })?;

    println!("Added {} notification channels", notification_manager.get_channels().len());
    println!();

    // 5. Record metrics and evaluate alerts
    println!("5. Recording metrics and evaluating alerts...");

    // Record CPU usage
    collector.record("cpu_usage", sys_metrics.cpu_usage_percent)?;
    let cpu_metric = collector.get_metric("cpu_usage")?;
    if let Some(data_point) = cpu_metric.data_points.last() {
        let alerts = alert_manager.evaluate("cpu_usage", data_point);
        if !alerts.is_empty() {
            println!("  CPU alert fired!");
            for alert in alerts {
                notification_manager.send_alert(&alert).await?;
            }
        } else {
            println!("  CPU usage normal ({:.2}%)", sys_metrics.cpu_usage_percent);
        }
    }

    // Record memory usage
    collector.record("memory_usage", sys_metrics.memory_usage_percent)?;
    let memory_metric = collector.get_metric("memory_usage")?;
    if let Some(data_point) = memory_metric.data_points.last() {
        let alerts = alert_manager.evaluate("memory_usage", data_point);
        if !alerts.is_empty() {
            println!("  Memory alert fired!");
            for alert in alerts {
                notification_manager.send_alert(&alert).await?;
            }
        } else {
            println!("  Memory usage normal ({:.2}%)", sys_metrics.memory_usage_percent);
        }
    }
    println!();

    // 6. Show active alerts
    println!("6. Active alerts:");
    let active_alerts = alert_manager.get_active_alerts();
    if active_alerts.is_empty() {
        println!("  No active alerts");
    } else {
        for alert in active_alerts {
            println!("  - [{}] {} (value: {:.2}, threshold: {:.2})",
                match alert.severity {
                    AlertSeverity::Info => "INFO",
                    AlertSeverity::Warning => "WARN",
                    AlertSeverity::Error => "ERROR",
                    AlertSeverity::Critical => "CRIT",
                },
                alert.rule_name,
                alert.metric_value,
                alert.threshold
            );
        }
    }
    println!();

    // 7. Simulate monitoring loop
    println!("7. Simulating monitoring loop (3 iterations)...");
    for i in 1..=3 {
        sleep(Duration::from_secs(2)).await;
        println!("\n  Iteration {}:", i);

        let sys_metrics = collector.collect_system_metrics();

        // Record and evaluate CPU
        collector.record("cpu_usage", sys_metrics.cpu_usage_percent)?;
        let cpu_metric = collector.get_metric("cpu_usage")?;
        if let Some(data_point) = cpu_metric.data_points.last() {
            let alerts = alert_manager.evaluate("cpu_usage", data_point);
            if !alerts.is_empty() {
                println!("    New CPU alerts: {}", alerts.len());
            }
        }

        // Record and evaluate memory
        collector.record("memory_usage", sys_metrics.memory_usage_percent)?;
        let memory_metric = collector.get_metric("memory_usage")?;
        if let Some(data_point) = memory_metric.data_points.last() {
            let alerts = alert_manager.evaluate("memory_usage", data_point);
            if !alerts.is_empty() {
                println!("    New memory alerts: {}", alerts.len());
            }
        }

        println!("    CPU: {:.2}%, Memory: {:.2}%",
            sys_metrics.cpu_usage_percent,
            sys_metrics.memory_usage_percent
        );
    }

    println!("\n=== Demo Complete ===");
    println!("Total notifications sent: {}", notification_manager.get_history().len());
    Ok(())
}
