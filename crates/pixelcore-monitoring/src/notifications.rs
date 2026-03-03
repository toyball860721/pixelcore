use crate::models::{Alert, AlertNotification, NotificationChannel};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct NotificationManager {
    channels: Arc<Mutex<Vec<NotificationChannel>>>,
    history: Arc<Mutex<Vec<AlertNotification>>>,
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(Mutex::new(Vec::new())),
            history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add a notification channel
    pub fn add_channel(&self, channel: NotificationChannel) -> Result<(), String> {
        let mut channels = self.channels.lock().unwrap();
        channels.push(channel);
        Ok(())
    }

    /// Remove all channels of a specific type
    pub fn remove_channels(&self, channel_type: &str) -> Result<(), String> {
        let mut channels = self.channels.lock().unwrap();
        channels.retain(|ch| {
            match (channel_type, ch) {
                ("email", NotificationChannel::Email { .. }) => false,
                ("slack", NotificationChannel::Slack { .. }) => false,
                ("webhook", NotificationChannel::Webhook { .. }) => false,
                ("console", NotificationChannel::Console) => false,
                _ => true,
            }
        });
        Ok(())
    }

    /// Send alert to all configured channels
    pub async fn send_alert(&self, alert: &Alert) -> Result<(), String> {
        let channels = self.channels.lock().unwrap().clone();

        for channel in channels {
            let mut notification = AlertNotification::new(alert.id, channel.clone());

            match self.send_to_channel(&channel, alert).await {
                Ok(_) => {
                    notification.success = true;
                    println!("✓ Alert sent via {:?}", channel);
                }
                Err(e) => {
                    notification.success = false;
                    notification.error_message = Some(e.clone());
                    eprintln!("✗ Failed to send alert via {:?}: {}", channel, e);
                }
            }

            let mut history = self.history.lock().unwrap();
            history.push(notification);
        }

        Ok(())
    }

    /// Send alert to a specific channel
    async fn send_to_channel(&self, channel: &NotificationChannel, alert: &Alert) -> Result<(), String> {
        match channel {
            NotificationChannel::Email { recipients } => {
                self.send_email(recipients, alert).await
            }
            NotificationChannel::Slack { webhook_url, channel: slack_channel } => {
                self.send_slack(webhook_url, slack_channel, alert).await
            }
            NotificationChannel::Webhook { url } => {
                self.send_webhook(url, alert).await
            }
            NotificationChannel::Console => {
                self.send_console(alert).await
            }
        }
    }

    /// Send email notification (mock implementation)
    async fn send_email(&self, recipients: &[String], alert: &Alert) -> Result<(), String> {
        println!("📧 Sending email to: {}", recipients.join(", "));
        println!("   Subject: [{}] {}",
            match alert.severity {
                crate::models::AlertSeverity::Info => "INFO",
                crate::models::AlertSeverity::Warning => "WARNING",
                crate::models::AlertSeverity::Error => "ERROR",
                crate::models::AlertSeverity::Critical => "CRITICAL",
            },
            alert.rule_name
        );
        println!("   Body: {}", alert.message);
        Ok(())
    }

    /// Send Slack notification (mock implementation)
    async fn send_slack(&self, webhook_url: &str, slack_channel: &str, alert: &Alert) -> Result<(), String> {
        println!("💬 Sending Slack notification");
        println!("   Webhook: {}", webhook_url);
        println!("   Channel: #{}", slack_channel);
        println!("   Message: [{}] {} - {}",
            match alert.severity {
                crate::models::AlertSeverity::Info => "ℹ️",
                crate::models::AlertSeverity::Warning => "⚠️",
                crate::models::AlertSeverity::Error => "❌",
                crate::models::AlertSeverity::Critical => "🚨",
            },
            alert.rule_name,
            alert.message
        );
        Ok(())
    }

    /// Send webhook notification (mock implementation)
    async fn send_webhook(&self, url: &str, alert: &Alert) -> Result<(), String> {
        println!("🔗 Sending webhook to: {}", url);
        println!("   Payload: {{");
        println!("     \"alert_id\": \"{}\",", alert.id);
        println!("     \"rule_name\": \"{}\",", alert.rule_name);
        println!("     \"severity\": \"{:?}\",", alert.severity);
        println!("     \"message\": \"{}\"", alert.message);
        println!("   }}");
        Ok(())
    }

    /// Send console notification
    async fn send_console(&self, alert: &Alert) -> Result<(), String> {
        println!("🖥️  Console Alert:");
        println!("   Rule: {}", alert.rule_name);
        println!("   Severity: {:?}", alert.severity);
        println!("   Status: {:?}", alert.status);
        println!("   Message: {}", alert.message);
        println!("   Value: {:.2} (threshold: {:.2})", alert.metric_value, alert.threshold);
        println!("   Fired at: {}", alert.fired_at.format("%Y-%m-%d %H:%M:%S"));
        Ok(())
    }

    /// Get all channels
    pub fn get_channels(&self) -> Vec<NotificationChannel> {
        let channels = self.channels.lock().unwrap();
        channels.clone()
    }

    /// Get notification history
    pub fn get_history(&self) -> Vec<AlertNotification> {
        let history = self.history.lock().unwrap();
        history.clone()
    }

    /// Clear notification history
    pub fn clear_history(&self) {
        let mut history = self.history.lock().unwrap();
        history.clear();
    }
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AlertRule, AlertCondition, AlertSeverity};

    #[tokio::test]
    async fn test_add_channel() {
        let manager = NotificationManager::new();

        let channel = NotificationChannel::Email {
            recipients: vec!["admin@example.com".to_string()],
        };

        assert!(manager.add_channel(channel).is_ok());
        assert_eq!(manager.get_channels().len(), 1);
    }

    #[tokio::test]
    async fn test_send_alert() {
        let manager = NotificationManager::new();

        let channel = NotificationChannel::Console;
        manager.add_channel(channel).unwrap();

        let rule = AlertRule::new(
            "High CPU Usage".to_string(),
            "CPU usage is above threshold".to_string(),
            "cpu_usage".to_string(),
            AlertCondition::GreaterThan,
            80.0,
            AlertSeverity::Warning,
        );

        let alert = Alert::new(&rule, 85.0, "CPU usage is high".to_string());

        assert!(manager.send_alert(&alert).await.is_ok());
        assert_eq!(manager.get_history().len(), 1);
    }

    #[tokio::test]
    async fn test_multiple_channels() {
        let manager = NotificationManager::new();

        manager.add_channel(NotificationChannel::Console).unwrap();
        manager.add_channel(NotificationChannel::Email {
            recipients: vec!["ops@example.com".to_string()],
        }).unwrap();
        manager.add_channel(NotificationChannel::Slack {
            webhook_url: "https://hooks.slack.com/test".to_string(),
            channel: "alerts".to_string(),
        }).unwrap();

        let rule = AlertRule::new(
            "High Memory".to_string(),
            "Memory usage is critical".to_string(),
            "memory_usage".to_string(),
            AlertCondition::GreaterThan,
            90.0,
            AlertSeverity::Critical,
        );

        let alert = Alert::new(&rule, 95.0, "Memory usage is critical".to_string());

        assert!(manager.send_alert(&alert).await.is_ok());
        assert_eq!(manager.get_history().len(), 3); // One notification per channel
    }
}
