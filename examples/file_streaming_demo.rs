//! Large file streaming demo
//!
//! This example demonstrates how to process large files efficiently
//! using streaming I/O without loading the entire file into memory.

use pixelcore_runtime::{FileStreamReader, FileStreamWriter, stream_copy};
use tempfile::NamedTempFile;

/// Generate a large test file
async fn generate_large_file(path: &str, size_mb: usize) -> std::io::Result<()> {
    println!("Generating {}MB test file...", size_mb);

    let mut writer = FileStreamWriter::new(path).await?;

    // Generate data in chunks
    let chunk_size = 1024 * 1024; // 1MB chunks
    let chunk_data = vec![b'A'; chunk_size];

    for i in 0..size_mb {
        writer.write_chunk(&chunk_data).await?;
        if (i + 1) % 10 == 0 {
            println!("  Generated {}MB...", i + 1);
        }
    }

    writer.flush().await?;
    println!("✓ File generated successfully\n");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Large File Streaming Demo ===\n");

    // Create temporary files
    let source_file = NamedTempFile::new()?;
    let dest_file = NamedTempFile::new()?;
    let source_path = source_file.path().to_str().unwrap();
    let dest_path = dest_file.path().to_str().unwrap();

    // Demo 1: Generate a large file (10MB)
    println!("Demo 1: Generating large file");
    println!("─────────────────────────────");
    generate_large_file(source_path, 10).await?;

    // Demo 2: Read file info
    println!("Demo 2: File information");
    println!("─────────────────────────────");
    let reader = FileStreamReader::new(source_path).await?;
    println!("File size: {} bytes ({:.2} MB)", reader.total_size(), reader.total_size() as f64 / 1024.0 / 1024.0);
    println!("Chunk size: {} bytes\n", pixelcore_runtime::DEFAULT_CHUNK_SIZE);

    // Demo 3: Stream copy with progress
    println!("Demo 3: Streaming file copy");
    println!("─────────────────────────────");
    println!("Copying file with progress tracking...");

    let start = std::time::Instant::now();
    let bytes_copied = stream_copy(source_path, dest_path, Some(1024 * 1024)).await?;
    let duration = start.elapsed();

    println!("✓ Copied {} bytes in {:.2}s", bytes_copied, duration.as_secs_f64());
    println!("  Speed: {:.2} MB/s\n", bytes_copied as f64 / 1024.0 / 1024.0 / duration.as_secs_f64());

    // Demo 4: Process file with custom logic
    println!("Demo 4: Custom file processing");
    println!("─────────────────────────────");
    println!("Processing file chunks...");

    let mut reader = FileStreamReader::new(source_path)
        .await?
        .with_chunk_size(1024 * 1024);

    let mut chunk_count = 0;
    let mut total_bytes = 0;

    reader.process_chunks(|chunk| async move {
        chunk_count += 1;
        total_bytes += chunk.len();
        if chunk_count % 5 == 0 {
            println!("  Processed {} chunks ({} bytes)", chunk_count, total_bytes);
        }
        Ok::<(), std::io::Error>(())
    }).await?;

    println!("✓ Processed {} chunks, {} total bytes\n", chunk_count, total_bytes);

    // Demo 5: Progress callback
    println!("Demo 5: Progress tracking");
    println!("─────────────────────────────");
    println!("Reading file with progress callback...");

    let mut reader = FileStreamReader::new(source_path)
        .await?
        .with_chunk_size(1024 * 1024)
        .with_progress(|bytes_read, total_size| {
            let percent = (bytes_read as f64 / total_size as f64 * 100.0) as u8;
            if bytes_read == total_size {
                println!("  Progress: 100% - Complete!");
            } else if percent % 20 == 0 {
                println!("  Progress: {}%", percent);
            }
        });

    while let Some(_chunk) = reader.read_chunk().await? {}

    println!("\n=== Demo completed successfully ===");
    Ok(())
}
