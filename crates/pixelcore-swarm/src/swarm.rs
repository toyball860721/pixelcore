use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use pixelcore_runtime::agent::{Agent, AgentId};
use pixelcore_runtime::message::Message;
use crate::error::SwarmError;

pub struct Swarm {
    agents: Arc<RwLock<HashMap<AgentId, Arc<tokio::sync::Mutex<dyn Agent>>>>>,
}

impl Swarm {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add(&self, agent: Arc<tokio::sync::Mutex<dyn Agent>>) {
        let id = agent.lock().await.id();
        self.agents.write().await.insert(id, agent);
    }

    pub async fn remove(&self, id: &AgentId) -> bool {
        self.agents.write().await.remove(id).is_some()
    }

    pub async fn get(&self, id: &AgentId) -> Result<Arc<tokio::sync::Mutex<dyn Agent>>, SwarmError> {
        self.agents
            .read()
            .await
            .get(id)
            .cloned()
            .ok_or_else(|| SwarmError::AgentNotFound(id.to_string()))
    }

    pub async fn count(&self) -> usize {
        self.agents.read().await.len()
    }

    pub async fn ids(&self) -> Vec<AgentId> {
        self.agents.read().await.keys().cloned().collect()
    }

    /// Send a message to a specific agent and return its reply.
    pub async fn route(&self, id: &AgentId, message: Message) -> Result<Message, SwarmError> {
        let agent = self.get(id).await?;
        let reply = agent.lock().await.process(message).await?;
        Ok(reply)
    }

    /// Broadcast a message to all agents concurrently; returns (id, reply) pairs, skipping errors.
    pub async fn broadcast(&self, message: Message) -> Vec<(AgentId, Message)> {
        let ids = self.ids().await;
        let handles: Vec<_> = ids.into_iter().map(|id| {
            let agents = Arc::clone(&self.agents);
            let msg = message.clone();
            tokio::spawn(async move {
                let agent = agents.read().await.get(&id).cloned()?;
                let reply = agent.lock().await.process(msg).await.ok()?;
                Some((id, reply))
            })
        }).collect();

        let mut results = Vec::new();
        for handle in handles {
            if let Ok(Some(pair)) = handle.await {
                results.push(pair);
            }
        }
        results
    }
}

impl Default for Swarm {
    fn default() -> Self {
        Self::new()
    }
}
