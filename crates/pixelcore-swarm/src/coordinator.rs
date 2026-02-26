use crate::swarm::Swarm;
use crate::error::SwarmError;
use pixelcore_runtime::event::{Event, EventBus, EventKind};

pub struct Coordinator {
    swarm: Swarm,
    event_bus: EventBus,
}

impl Coordinator {
    pub fn new(swarm: Swarm, event_bus: EventBus) -> Self {
        Self { swarm, event_bus }
    }

    pub async fn broadcast(&self, payload: serde_json::Value) -> Result<(), SwarmError> {
        let ids = self.swarm.ids().await;
        for _id in ids {
            let event = Event::new(EventKind::Custom("agent_message".to_string()), "coordinator", payload.clone());
            let _ = self.event_bus.publish(event);
        }
        Ok(())
    }

    pub fn swarm(&self) -> &Swarm {
        &self.swarm
    }
}
