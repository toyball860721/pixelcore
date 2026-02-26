use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use flume::{Sender, Receiver};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub kind: EventKind,
    pub source: String,
    pub timestamp: DateTime<Utc>,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    AgentStarted,
    AgentStopped,
    AgentError,
    MessageReceived,
    MessageSent,
    TaskStarted,
    TaskCompleted,
    TaskFailed,
    HeartbeatTick,
    Custom(String),
}

impl Event {
    pub fn new(kind: EventKind, source: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind,
            source: source.into(),
            timestamp: Utc::now(),
            payload,
        }
    }
}

#[derive(Clone)]
pub struct EventBus {
    sender: Sender<Event>,
    receiver: Receiver<Event>,
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, receiver) = flume::unbounded();
        Self { sender, receiver }
    }

    pub fn publish(&self, event: Event) -> Result<(), flume::SendError<Event>> {
        self.sender.send(event)
    }

    pub fn subscribe(&self) -> Receiver<Event> {
        self.receiver.clone()
    }

    pub fn sender(&self) -> Sender<Event> {
        self.sender.clone()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
