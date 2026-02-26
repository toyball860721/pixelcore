use std::time::Duration;
use tokio::time;
use pixelcore_runtime::event::{Event, EventBus, EventKind};

#[derive(Debug, Clone)]
pub struct HeartbeatConfig {
    pub interval: Duration,
    pub source: String,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(30),
            source: "heartbeat".to_string(),
        }
    }
}

pub struct Heartbeat {
    config: HeartbeatConfig,
    event_bus: EventBus,
}

impl Heartbeat {
    pub fn new(config: HeartbeatConfig, event_bus: EventBus) -> Self {
        Self { config, event_bus }
    }

    pub async fn run(&self) {
        let mut interval = time::interval(self.config.interval);
        loop {
            interval.tick().await;
            let event = Event::new(
                EventKind::HeartbeatTick,
                &self.config.source,
                serde_json::json!({ "ts": chrono::Utc::now().to_rfc3339() }),
            );
            let _ = self.event_bus.publish(event);
        }
    }
}
