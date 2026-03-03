use crate::models::{Alert, AlertRule, AlertStatus, MetricDataPoint};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AlertManager {
    rules: Arc<Mutex<HashMap<Uuid, AlertRule>>>,
    active_alerts: Arc<Mutex<HashMap<Uuid, Alert>>>,
}

impl AlertManager {
    pub fn new() -> Self {
        Self {
            rules: Arc::new(Mutex::new(HashMap::new())),
            active_alerts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Add or update an alert rule
    pub fn add_rule(&self, rule: AlertRule) -> Result<(), String> {
        let mut rules = self.rules.lock().unwrap();
        rules.insert(rule.id, rule);
        Ok(())
    }

    /// Remove an alert rule
    pub fn remove_rule(&self, rule_id: Uuid) -> Result<(), String> {
        let mut rules = self.rules.lock().unwrap();
        rules.remove(&rule_id)
            .ok_or_else(|| format!("Rule not found: {}", rule_id))?;
        Ok(())
    }

    /// Evaluate a metric data point against all rules
    pub fn evaluate(&self, metric_name: &str, data_point: &MetricDataPoint) -> Vec<Alert> {
        let rules = self.rules.lock().unwrap();
        let mut active_alerts = self.active_alerts.lock().unwrap();
        let mut new_alerts = Vec::new();

        for rule in rules.values() {
            if !rule.enabled || rule.metric_name != metric_name {
                continue;
            }

            if rule.evaluate(data_point.value) {
                // Create or update alert
                if !active_alerts.contains_key(&rule.id) {
                    let message = format!(
                        "{}: {} (value: {:.2}, threshold: {:.2})",
                        rule.name, rule.description, data_point.value, rule.threshold
                    );
                    let alert = Alert::new(rule, data_point.value, message);
                    active_alerts.insert(rule.id, alert.clone());
                    new_alerts.push(alert);
                }
            } else {
                // Resolve alert if it exists
                if let Some(alert) = active_alerts.get_mut(&rule.id) {
                    if alert.status == AlertStatus::Firing {
                        alert.resolve();
                        new_alerts.push(alert.clone());
                    }
                }
            }
        }

        new_alerts
    }

    /// Get all active alerts
    pub fn get_active_alerts(&self) -> Vec<Alert> {
        let alerts = self.active_alerts.lock().unwrap();
        alerts.values()
            .filter(|a| a.status == AlertStatus::Firing)
            .cloned()
            .collect()
    }

    /// Get all rules
    pub fn get_rules(&self) -> Vec<AlertRule> {
        let rules = self.rules.lock().unwrap();
        rules.values().cloned().collect()
    }

    /// Get a specific rule
    pub fn get_rule(&self, rule_id: Uuid) -> Option<AlertRule> {
        let rules = self.rules.lock().unwrap();
        rules.get(&rule_id).cloned()
    }

    /// Acknowledge an alert
    pub fn acknowledge_alert(&self, alert_id: Uuid, by: String) -> Result<(), String> {
        let mut alerts = self.active_alerts.lock().unwrap();
        let alert = alerts.get_mut(&alert_id)
            .ok_or_else(|| format!("Alert not found: {}", alert_id))?;
        alert.acknowledge(by);
        Ok(())
    }

    /// Clear resolved alerts older than specified duration
    pub fn cleanup_resolved(&self, max_age_secs: i64) {
        let mut alerts = self.active_alerts.lock().unwrap();
        let now = chrono::Utc::now();
        alerts.retain(|_, alert| {
            if alert.status == AlertStatus::Resolved {
                if let Some(resolved_at) = alert.resolved_at {
                    (now - resolved_at).num_seconds() < max_age_secs
                } else {
                    true
                }
            } else {
                true
            }
        });
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AlertCondition, AlertSeverity};

    #[test]
    fn test_add_and_remove_rule() {
        let manager = AlertManager::new();

        let rule = AlertRule::new(
            "High CPU Usage".to_string(),
            "CPU usage is above threshold".to_string(),
            "cpu_usage".to_string(),
            AlertCondition::GreaterThan,
            80.0,
            AlertSeverity::Warning,
        );

        let rule_id = rule.id;
        assert!(manager.add_rule(rule).is_ok());
        assert_eq!(manager.get_rules().len(), 1);

        assert!(manager.remove_rule(rule_id).is_ok());
        assert_eq!(manager.get_rules().len(), 0);
    }

    #[test]
    fn test_evaluate_firing_alert() {
        let manager = AlertManager::new();

        let rule = AlertRule::new(
            "High CPU Usage".to_string(),
            "CPU usage is above threshold".to_string(),
            "cpu_usage".to_string(),
            AlertCondition::GreaterThan,
            80.0,
            AlertSeverity::Warning,
        );
        manager.add_rule(rule).unwrap();

        let data_point = MetricDataPoint::new(85.0);
        let alerts = manager.evaluate("cpu_usage", &data_point);

        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].status, AlertStatus::Firing);
    }

    #[test]
    fn test_evaluate_resolved_alert() {
        let manager = AlertManager::new();

        let rule = AlertRule::new(
            "High CPU Usage".to_string(),
            "CPU usage is above threshold".to_string(),
            "cpu_usage".to_string(),
            AlertCondition::GreaterThan,
            80.0,
            AlertSeverity::Warning,
        );
        manager.add_rule(rule).unwrap();

        // First evaluation - fire alert
        let data_point1 = MetricDataPoint::new(85.0);
        manager.evaluate("cpu_usage", &data_point1);

        // Second evaluation - resolve alert
        let data_point2 = MetricDataPoint::new(70.0);
        let alerts = manager.evaluate("cpu_usage", &data_point2);

        assert!(alerts.iter().any(|a| a.status == AlertStatus::Resolved));
    }
}
