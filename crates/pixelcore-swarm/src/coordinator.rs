use crate::swarm::Swarm;
use crate::error::SwarmError;
use pixelcore_runtime::agent::AgentId;
use pixelcore_runtime::event::{Event, EventBus, EventKind};
use pixelcore_runtime::message::Message;

pub struct Coordinator {
    swarm: Swarm,
    event_bus: EventBus,
}

impl Coordinator {
    pub fn new(swarm: Swarm, event_bus: EventBus) -> Self {
        Self { swarm, event_bus }
    }

    /// Route a message to a specific agent, publish result event.
    pub async fn route(&self, id: &AgentId, message: Message) -> Result<Message, SwarmError> {
        let reply = self.swarm.route(id, message).await?;
        let _ = self.event_bus.publish(Event::new(
            EventKind::MessageSent,
            "coordinator",
            serde_json::json!({ "agent": id.to_string(), "reply": reply.content }),
        ));
        Ok(reply)
    }

    /// Broadcast a message to all agents.
    pub async fn broadcast(&self, message: Message) -> Result<Vec<(AgentId, Message)>, SwarmError> {
        let results = self.swarm.broadcast(message).await;
        let _ = self.event_bus.publish(Event::new(
            EventKind::Custom("broadcast_complete".to_string()),
            "coordinator",
            serde_json::json!({ "replies": results.len() }),
        ));
        Ok(results)
    }

    pub fn swarm(&self) -> &Swarm {
        &self.swarm
    }

    pub fn event_bus(&self) -> &EventBus {
        &self.event_bus
    }
}
