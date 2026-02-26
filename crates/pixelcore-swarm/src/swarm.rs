use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use pixelcore_runtime::agent::{Agent, AgentId};
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
}

impl Default for Swarm {
    fn default() -> Self {
        Self::new()
    }
}
