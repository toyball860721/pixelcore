//! Streaming response demo
//!
//! This example demonstrates how to use streaming responses
//! to provide real-time feedback for long-running tasks.

use pixelcore_runtime::StreamingResponse;
use std::time::Duration;

/// Simulate a long-running task with streaming updates
async fn process_data_with_streaming(sender: pixelcore_runtime::StreamingSender) {
    // Send initial status
    sender.send_status("Starting data processing...").unwrap();
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Process in stages with progress updates
    let stages = vec![
        ("Loading data", 20),
        ("Validating data", 40),
        ("Transforming data", 60),
        ("Analyzing data", 80),
        ("Generating report", 100),
    ];

    for (stage, progress) in stages {
        sender.send_progress(progress, stage).unwrap();
        sender.send_text(format!("  - {}\n", stage)).unwrap();
        tokio::time::sleep(Duration::from_millis(300)).await;
    }

    // Send final results
    sender.send_text("\nResults:\n").unwrap();
    sender.send_text("  Total records: 1,234\n").unwrap();
    sender.send_text("  Valid records: 1,200\n").unwrap();
    sender.send_text("  Invalid records: 34\n").unwrap();

    // Send completion
    sender.send_status("Processing complete!").unwrap();
    sender.send_done().unwrap();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Streaming Response Demo ===\n");

    // Create streaming response
    let stream = StreamingResponse::new();
    let (sender, mut receiver) = stream.split();

    // Start the long-running task in background
    tokio::spawn(async move {
        process_data_with_streaming(sender).await;
    });

    // Receive and display chunks in real-time
    println!("Receiving streaming updates:\n");

    while let Some(chunk) = receiver.recv().await {
        match chunk.chunk_type {
            pixelcore_runtime::ChunkType::Text => {
                print!("{}", chunk.content);
            }
            pixelcore_runtime::ChunkType::Status => {
                println!("📢 {}", chunk.content);
            }
            pixelcore_runtime::ChunkType::Progress => {
                if let Some(metadata) = &chunk.metadata {
                    if let Some(percent) = metadata.get("percent") {
                        println!("⏳ [{}%] {}", percent, chunk.content);
                    }
                }
            }
            pixelcore_runtime::ChunkType::Error => {
                println!("❌ Error: {}", chunk.content);
            }
            pixelcore_runtime::ChunkType::Done => {
                println!("\n✅ Stream completed");
                break;
            }
        }
    }

    println!("\n=== Demo completed successfully ===");
    Ok(())
}
