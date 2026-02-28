use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// 消息总线事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusMessage {
    pub id: Uuid,
    pub from: Uuid,
    pub to: Option<Uuid>, // None = 广播
    pub topic: String,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

impl BusMessage {
    pub fn new(from: Uuid, to: Option<Uuid>, topic: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            from,
            to,
            topic: topic.into(),
            payload,
            timestamp: Utc::now(),
        }
    }

    pub fn broadcast(from: Uuid, topic: impl Into<String>, payload: serde_json::Value) -> Self {
        Self::new(from, None, topic, payload)
    }

    pub fn direct(from: Uuid, to: Uuid, topic: impl Into<String>, payload: serde_json::Value) -> Self {
        Self::new(from, Some(to), topic, payload)
    }
}

/// 消息订阅者
type Subscriber = mpsc::UnboundedSender<BusMessage>;

/// 消息总线
pub struct MessageBus {
    /// 主题订阅者: topic -> subscribers
    topic_subscribers: Arc<RwLock<HashMap<String, Vec<Subscriber>>>>,
    /// Agent 订阅者: agent_id -> subscriber
    agent_subscribers: Arc<RwLock<HashMap<Uuid, Subscriber>>>,
}

impl MessageBus {
    pub fn new() -> Self {
        Self {
            topic_subscribers: Arc::new(RwLock::new(HashMap::new())),
            agent_subscribers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 订阅主题
    pub async fn subscribe_topic(&self, topic: impl Into<String>) -> mpsc::UnboundedReceiver<BusMessage> {
        let topic = topic.into();
        let (tx, rx) = mpsc::unbounded_channel();

        let mut subscribers = self.topic_subscribers.write().await;
        subscribers.entry(topic).or_insert_with(Vec::new).push(tx);

        rx
    }

    /// 订阅 Agent 的直接消息
    pub async fn subscribe_agent(&self, agent_id: Uuid) -> mpsc::UnboundedReceiver<BusMessage> {
        let (tx, rx) = mpsc::unbounded_channel();

        let mut subscribers = self.agent_subscribers.write().await;
        subscribers.insert(agent_id, tx);

        rx
    }

    /// 取消订阅主题
    pub async fn unsubscribe_topic(&self, topic: &str) {
        let mut subscribers = self.topic_subscribers.write().await;
        subscribers.remove(topic);
    }

    /// 取消订阅 Agent
    pub async fn unsubscribe_agent(&self, agent_id: &Uuid) {
        let mut subscribers = self.agent_subscribers.write().await;
        subscribers.remove(agent_id);
    }

    /// 发布消息
    pub async fn publish(&self, message: BusMessage) {
        // 如果是直接消息，发送给特定 Agent
        if let Some(to) = message.to {
            let subscribers = self.agent_subscribers.read().await;
            if let Some(subscriber) = subscribers.get(&to) {
                let _ = subscriber.send(message.clone());
            }
        } else {
            // 广播消息，发送给所有订阅该主题的 Agent
            let subscribers = self.topic_subscribers.read().await;
            if let Some(subs) = subscribers.get(&message.topic) {
                for subscriber in subs {
                    let _ = subscriber.send(message.clone());
                }
            }
        }
    }

    /// 获取主题订阅者数量
    pub async fn topic_subscriber_count(&self, topic: &str) -> usize {
        let subscribers = self.topic_subscribers.read().await;
        subscribers.get(topic).map(|s| s.len()).unwrap_or(0)
    }

    /// 获取 Agent 订阅者数量
    pub async fn agent_subscriber_count(&self) -> usize {
        let subscribers = self.agent_subscribers.read().await;
        subscribers.len()
    }
}

impl Default for MessageBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_topic_subscribe_and_publish() {
        let bus = MessageBus::new();
        let mut rx = bus.subscribe_topic("test.topic").await;

        let from = Uuid::new_v4();
        let message = BusMessage::broadcast(from, "test.topic", serde_json::json!({"data": "hello"}));

        bus.publish(message.clone()).await;

        let received = rx.recv().await.unwrap();
        assert_eq!(received.id, message.id);
        assert_eq!(received.topic, "test.topic");
    }

    #[tokio::test]
    async fn test_direct_message() {
        let bus = MessageBus::new();
        let agent_id = Uuid::new_v4();
        let mut rx = bus.subscribe_agent(agent_id).await;

        let from = Uuid::new_v4();
        let message = BusMessage::direct(from, agent_id, "direct.message", serde_json::json!({"data": "hello"}));

        bus.publish(message.clone()).await;

        let received = rx.recv().await.unwrap();
        assert_eq!(received.id, message.id);
        assert_eq!(received.to, Some(agent_id));
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let bus = MessageBus::new();
        let mut rx1 = bus.subscribe_topic("test.topic").await;
        let mut rx2 = bus.subscribe_topic("test.topic").await;

        let from = Uuid::new_v4();
        let message = BusMessage::broadcast(from, "test.topic", serde_json::json!({"data": "hello"}));

        bus.publish(message.clone()).await;

        let received1 = rx1.recv().await.unwrap();
        let received2 = rx2.recv().await.unwrap();

        assert_eq!(received1.id, message.id);
        assert_eq!(received2.id, message.id);
    }
}
