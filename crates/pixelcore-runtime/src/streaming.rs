//! Streaming response support for real-time output
//!
//! This module provides streaming response capabilities that allow
//! agents to send responses incrementally, improving user experience
//! for long-running tasks.

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use std::fmt;

/// Type of response chunk
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChunkType {
    /// Text content
    Text,
    /// Status update
    Status,
    /// Progress update (0-100)
    Progress,
    /// Error message
    Error,
    /// Completion signal
    Done,
}

/// A chunk of streaming response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseChunk {
    /// Type of this chunk
    pub chunk_type: ChunkType,
    /// Content of the chunk
    pub content: String,
    /// Optional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl ResponseChunk {
    /// Create a text chunk
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            chunk_type: ChunkType::Text,
            content: content.into(),
            metadata: None,
        }
    }

    /// Create a status chunk
    pub fn status(content: impl Into<String>) -> Self {
        Self {
            chunk_type: ChunkType::Status,
            content: content.into(),
            metadata: None,
        }
    }

    /// Create a progress chunk
    pub fn progress(percent: u8, message: impl Into<String>) -> Self {
        let metadata = serde_json::json!({ "percent": percent });
        Self {
            chunk_type: ChunkType::Progress,
            content: message.into(),
            metadata: Some(metadata),
        }
    }

    /// Create an error chunk
    pub fn error(content: impl Into<String>) -> Self {
        Self {
            chunk_type: ChunkType::Error,
            content: content.into(),
            metadata: None,
        }
    }

    /// Create a done chunk
    pub fn done() -> Self {
        Self {
            chunk_type: ChunkType::Done,
            content: String::new(),
            metadata: None,
        }
    }
}

impl fmt::Display for ResponseChunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.chunk_type {
            ChunkType::Text => write!(f, "{}", self.content),
            ChunkType::Status => write!(f, "[STATUS] {}", self.content),
            ChunkType::Progress => {
                if let Some(metadata) = &self.metadata {
                    if let Some(percent) = metadata.get("percent") {
                        write!(f, "[PROGRESS {}%] {}", percent, self.content)
                    } else {
                        write!(f, "[PROGRESS] {}", self.content)
                    }
                } else {
                    write!(f, "[PROGRESS] {}", self.content)
                }
            }
            ChunkType::Error => write!(f, "[ERROR] {}", self.content),
            ChunkType::Done => write!(f, "[DONE]"),
        }
    }
}

/// Sender for streaming responses
pub struct StreamingSender {
    tx: mpsc::UnboundedSender<ResponseChunk>,
}

impl StreamingSender {
    /// Send a text chunk
    pub fn send_text(&self, content: impl Into<String>) -> Result<(), String> {
        self.tx
            .send(ResponseChunk::text(content))
            .map_err(|e| format!("Failed to send text: {}", e))
    }

    /// Send a status chunk
    pub fn send_status(&self, content: impl Into<String>) -> Result<(), String> {
        self.tx
            .send(ResponseChunk::status(content))
            .map_err(|e| format!("Failed to send status: {}", e))
    }

    /// Send a progress chunk
    pub fn send_progress(&self, percent: u8, message: impl Into<String>) -> Result<(), String> {
        self.tx
            .send(ResponseChunk::progress(percent, message))
            .map_err(|e| format!("Failed to send progress: {}", e))
    }

    /// Send an error chunk
    pub fn send_error(&self, content: impl Into<String>) -> Result<(), String> {
        self.tx
            .send(ResponseChunk::error(content))
            .map_err(|e| format!("Failed to send error: {}", e))
    }

    /// Send a done chunk
    pub fn send_done(&self) -> Result<(), String> {
        self.tx
            .send(ResponseChunk::done())
            .map_err(|e| format!("Failed to send done: {}", e))
    }

    /// Send a custom chunk
    pub fn send_chunk(&self, chunk: ResponseChunk) -> Result<(), String> {
        self.tx
            .send(chunk)
            .map_err(|e| format!("Failed to send chunk: {}", e))
    }
}

/// Receiver for streaming responses
pub struct StreamingReceiver {
    rx: mpsc::UnboundedReceiver<ResponseChunk>,
}

impl StreamingReceiver {
    /// Receive the next chunk
    pub async fn recv(&mut self) -> Option<ResponseChunk> {
        self.rx.recv().await
    }

    /// Try to receive a chunk without blocking
    pub fn try_recv(&mut self) -> Result<ResponseChunk, mpsc::error::TryRecvError> {
        self.rx.try_recv()
    }

    /// Collect all remaining chunks
    pub async fn collect_all(&mut self) -> Vec<ResponseChunk> {
        let mut chunks = Vec::new();
        while let Some(chunk) = self.recv().await {
            chunks.push(chunk);
        }
        chunks
    }

    /// Collect all text chunks into a single string
    pub async fn collect_text(&mut self) -> String {
        let mut result = String::new();
        while let Some(chunk) = self.recv().await {
            if chunk.chunk_type == ChunkType::Text {
                result.push_str(&chunk.content);
            }
        }
        result
    }
}

/// Streaming response that provides both sender and receiver
pub struct StreamingResponse {
    sender: StreamingSender,
    receiver: StreamingReceiver,
}

impl StreamingResponse {
    /// Create a new streaming response
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            sender: StreamingSender { tx },
            receiver: StreamingReceiver { rx },
        }
    }

    /// Split into sender and receiver
    pub fn split(self) -> (StreamingSender, StreamingReceiver) {
        (self.sender, self.receiver)
    }

    /// Get a reference to the sender
    pub fn sender(&self) -> &StreamingSender {
        &self.sender
    }

    /// Get a mutable reference to the receiver
    pub fn receiver_mut(&mut self) -> &mut StreamingReceiver {
        &mut self.receiver
    }
}

impl Default for StreamingResponse {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_streaming_basic() {
        let stream = StreamingResponse::new();
        let (sender, mut receiver) = stream.split();

        // Send some chunks
        sender.send_text("Hello").unwrap();
        sender.send_text(" World").unwrap();
        sender.send_done().unwrap();

        // Receive chunks
        let chunk1 = receiver.recv().await.unwrap();
        assert_eq!(chunk1.chunk_type, ChunkType::Text);
        assert_eq!(chunk1.content, "Hello");

        let chunk2 = receiver.recv().await.unwrap();
        assert_eq!(chunk2.chunk_type, ChunkType::Text);
        assert_eq!(chunk2.content, " World");

        let chunk3 = receiver.recv().await.unwrap();
        assert_eq!(chunk3.chunk_type, ChunkType::Done);
    }

    #[tokio::test]
    async fn test_streaming_progress() {
        let stream = StreamingResponse::new();
        let (sender, mut receiver) = stream.split();

        // Send progress updates
        sender.send_progress(0, "Starting").unwrap();
        sender.send_progress(50, "Half done").unwrap();
        sender.send_progress(100, "Complete").unwrap();

        // Receive and verify
        let chunk1 = receiver.recv().await.unwrap();
        assert_eq!(chunk1.chunk_type, ChunkType::Progress);
        assert_eq!(chunk1.content, "Starting");

        let chunk2 = receiver.recv().await.unwrap();
        assert_eq!(chunk2.chunk_type, ChunkType::Progress);
        assert_eq!(chunk2.content, "Half done");

        let chunk3 = receiver.recv().await.unwrap();
        assert_eq!(chunk3.chunk_type, ChunkType::Progress);
        assert_eq!(chunk3.content, "Complete");
    }

    #[tokio::test]
    async fn test_streaming_collect_text() {
        let stream = StreamingResponse::new();
        let (sender, mut receiver) = stream.split();

        // Send text chunks
        tokio::spawn(async move {
            sender.send_text("Hello").unwrap();
            sender.send_status("Processing").unwrap();
            sender.send_text(" World").unwrap();
            sender.send_text("!").unwrap();
            sender.send_done().unwrap();
        });

        // Collect all text
        let text = receiver.collect_text().await;
        assert_eq!(text, "Hello World!");
    }

    #[tokio::test]
    async fn test_streaming_status_and_error() {
        let stream = StreamingResponse::new();
        let (sender, mut receiver) = stream.split();

        // Send different types
        sender.send_status("Initializing").unwrap();
        sender.send_error("Something went wrong").unwrap();
        sender.send_done().unwrap();

        // Verify types
        let chunk1 = receiver.recv().await.unwrap();
        assert_eq!(chunk1.chunk_type, ChunkType::Status);

        let chunk2 = receiver.recv().await.unwrap();
        assert_eq!(chunk2.chunk_type, ChunkType::Error);
        assert_eq!(chunk2.content, "Something went wrong");

        let chunk3 = receiver.recv().await.unwrap();
        assert_eq!(chunk3.chunk_type, ChunkType::Done);
    }
}
