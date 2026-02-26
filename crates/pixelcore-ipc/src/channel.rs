use serde::{Deserialize, Serialize};
use uuid::Uuid;
use flume::{Sender, Receiver};
use crate::error::IpcError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcMessage {
    pub id: Uuid,
    pub from: String,
    pub to: String,
    pub payload: serde_json::Value,
}

impl IpcMessage {
    pub fn new(from: impl Into<String>, to: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            from: from.into(),
            to: to.into(),
            payload,
        }
    }
}

#[derive(Clone)]
pub struct IpcChannel {
    sender: Sender<IpcMessage>,
    receiver: Receiver<IpcMessage>,
}

impl IpcChannel {
    pub fn new() -> Self {
        let (sender, receiver) = flume::unbounded();
        Self { sender, receiver }
    }

    pub fn send(&self, msg: IpcMessage) -> Result<(), IpcError> {
        self.sender.send(msg).map_err(|e| IpcError::Send(e.to_string()))
    }

    pub fn recv(&self) -> Result<IpcMessage, IpcError> {
        self.receiver.recv().map_err(|_| IpcError::ChannelClosed)
    }

    pub fn try_recv(&self) -> Option<IpcMessage> {
        self.receiver.try_recv().ok()
    }

    pub fn sender(&self) -> Sender<IpcMessage> {
        self.sender.clone()
    }

    pub fn receiver(&self) -> Receiver<IpcMessage> {
        self.receiver.clone()
    }
}

impl Default for IpcChannel {
    fn default() -> Self {
        Self::new()
    }
}
