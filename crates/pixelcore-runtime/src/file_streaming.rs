//! Large file streaming support for memory-efficient file processing
//!
//! This module provides streaming file I/O capabilities that allow
//! processing large files without loading them entirely into memory.

use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use std::io::Result as IoResult;

/// Default chunk size for streaming (1MB)
pub const DEFAULT_CHUNK_SIZE: usize = 1024 * 1024;

/// Progress callback type
pub type ProgressCallback = Box<dyn Fn(u64, u64) + Send + Sync>;

/// File stream reader for processing large files in chunks
pub struct FileStreamReader {
    file: BufReader<File>,
    chunk_size: usize,
    total_size: u64,
    bytes_read: u64,
    progress_callback: Option<ProgressCallback>,
}

impl FileStreamReader {
    /// Create a new file stream reader
    pub async fn new(path: impl AsRef<Path>) -> IoResult<Self> {
        let file = File::open(path).await?;
        let metadata = file.metadata().await?;
        let total_size = metadata.len();

        Ok(Self {
            file: BufReader::new(file),
            chunk_size: DEFAULT_CHUNK_SIZE,
            total_size,
            bytes_read: 0,
            progress_callback: None,
        })
    }

    /// Set custom chunk size
    pub fn with_chunk_size(mut self, chunk_size: usize) -> Self {
        self.chunk_size = chunk_size;
        self
    }

    /// Set progress callback
    pub fn with_progress<F>(mut self, callback: F) -> Self
    where
        F: Fn(u64, u64) + Send + Sync + 'static,
    {
        self.progress_callback = Some(Box::new(callback));
        self
    }

    /// Get total file size
    pub fn total_size(&self) -> u64 {
        self.total_size
    }

    /// Get bytes read so far
    pub fn bytes_read(&self) -> u64 {
        self.bytes_read
    }

    /// Get progress percentage (0-100)
    pub fn progress_percent(&self) -> u8 {
        if self.total_size == 0 {
            100
        } else {
            ((self.bytes_read as f64 / self.total_size as f64) * 100.0) as u8
        }
    }

    /// Read next chunk from file
    pub async fn read_chunk(&mut self) -> IoResult<Option<Vec<u8>>> {
        let mut buffer = vec![0u8; self.chunk_size];
        let bytes_read = self.file.read(&mut buffer).await?;

        if bytes_read == 0 {
            return Ok(None);
        }

        buffer.truncate(bytes_read);
        self.bytes_read += bytes_read as u64;

        // Call progress callback if set
        if let Some(callback) = &self.progress_callback {
            callback(self.bytes_read, self.total_size);
        }

        Ok(Some(buffer))
    }

    /// Process all chunks with a callback
    pub async fn process_chunks<F, Fut, E>(&mut self, mut processor: F) -> Result<(), E>
    where
        F: FnMut(Vec<u8>) -> Fut,
        Fut: std::future::Future<Output = Result<(), E>>,
        E: From<std::io::Error>,
    {
        while let Some(chunk) = self.read_chunk().await? {
            processor(chunk).await?;
        }
        Ok(())
    }
}

/// File stream writer for writing large files in chunks
pub struct FileStreamWriter {
    file: BufWriter<File>,
    chunk_size: usize,
    bytes_written: u64,
    progress_callback: Option<ProgressCallback>,
}

impl FileStreamWriter {
    /// Create a new file stream writer
    pub async fn new(path: impl AsRef<Path>) -> IoResult<Self> {
        let file = File::create(path).await?;

        Ok(Self {
            file: BufWriter::new(file),
            chunk_size: DEFAULT_CHUNK_SIZE,
            bytes_written: 0,
            progress_callback: None,
        })
    }

    /// Set custom chunk size
    pub fn with_chunk_size(mut self, chunk_size: usize) -> Self {
        self.chunk_size = chunk_size;
        self
    }

    /// Set progress callback
    pub fn with_progress<F>(mut self, callback: F) -> Self
    where
        F: Fn(u64, u64) + Send + Sync + 'static,
    {
        self.progress_callback = Some(Box::new(callback));
        self
    }

    /// Get bytes written so far
    pub fn bytes_written(&self) -> u64 {
        self.bytes_written
    }

    /// Write a chunk to file
    pub async fn write_chunk(&mut self, data: &[u8]) -> IoResult<()> {
        self.file.write_all(data).await?;
        self.bytes_written += data.len() as u64;

        // Call progress callback if set
        if let Some(callback) = &self.progress_callback {
            callback(self.bytes_written, 0); // Total size unknown for writer
        }

        Ok(())
    }

    /// Flush the buffer
    pub async fn flush(&mut self) -> IoResult<()> {
        self.file.flush().await
    }
}

/// Copy a file in streaming mode
pub async fn stream_copy(
    source: impl AsRef<Path>,
    dest: impl AsRef<Path>,
    chunk_size: Option<usize>,
) -> IoResult<u64> {
    let mut reader = FileStreamReader::new(source).await?;
    if let Some(size) = chunk_size {
        reader = reader.with_chunk_size(size);
    }

    let mut writer = FileStreamWriter::new(dest).await?;
    if let Some(size) = chunk_size {
        writer = writer.with_chunk_size(size);
    }

    while let Some(chunk) = reader.read_chunk().await? {
        writer.write_chunk(&chunk).await?;
    }

    writer.flush().await?;
    Ok(reader.bytes_read())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_file_stream_reader() {
        // Create a temporary file with test data
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_data = b"Hello, World! This is a test file for streaming.";
        temp_file.write_all(test_data).unwrap();
        temp_file.flush().unwrap();

        // Read file in chunks
        let mut reader = FileStreamReader::new(temp_file.path())
            .await
            .unwrap()
            .with_chunk_size(10);

        let mut all_data = Vec::new();
        while let Some(chunk) = reader.read_chunk().await.unwrap() {
            all_data.extend_from_slice(&chunk);
        }

        assert_eq!(all_data, test_data);
        assert_eq!(reader.bytes_read(), test_data.len() as u64);
        assert_eq!(reader.progress_percent(), 100);
    }

    #[tokio::test]
    async fn test_file_stream_writer() {
        let temp_file = NamedTempFile::new().unwrap();
        let test_data = b"Hello, World! This is a test file for streaming.";

        // Write file in chunks
        let mut writer = FileStreamWriter::new(temp_file.path())
            .await
            .unwrap()
            .with_chunk_size(10);

        for chunk in test_data.chunks(10) {
            writer.write_chunk(chunk).await.unwrap();
        }
        writer.flush().await.unwrap();

        assert_eq!(writer.bytes_written(), test_data.len() as u64);

        // Verify written data
        let written_data = tokio::fs::read(temp_file.path()).await.unwrap();
        assert_eq!(written_data, test_data);
    }

    #[tokio::test]
    async fn test_stream_copy() {
        // Create source file
        let mut source_file = NamedTempFile::new().unwrap();
        let test_data = b"Hello, World! This is a test file for streaming copy.";
        source_file.write_all(test_data).unwrap();
        source_file.flush().unwrap();

        // Create destination file
        let dest_file = NamedTempFile::new().unwrap();

        // Copy file
        let bytes_copied = stream_copy(source_file.path(), dest_file.path(), Some(10))
            .await
            .unwrap();

        assert_eq!(bytes_copied, test_data.len() as u64);

        // Verify copied data
        let copied_data = tokio::fs::read(dest_file.path()).await.unwrap();
        assert_eq!(copied_data, test_data);
    }

    #[tokio::test]
    async fn test_progress_callback() {
        // Create a temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_data = vec![0u8; 1000]; // 1KB of data
        temp_file.write_all(&test_data).unwrap();
        temp_file.flush().unwrap();

        // Track progress
        let progress_calls = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let progress_calls_clone = progress_calls.clone();

        let mut reader = FileStreamReader::new(temp_file.path())
            .await
            .unwrap()
            .with_chunk_size(100)
            .with_progress(move |bytes_read, total_size| {
                progress_calls_clone
                    .lock()
                    .unwrap()
                    .push((bytes_read, total_size));
            });

        // Read all chunks
        while let Some(_chunk) = reader.read_chunk().await.unwrap() {}

        // Verify progress was tracked
        let calls = progress_calls.lock().unwrap();
        assert!(!calls.is_empty());
        assert_eq!(calls.last().unwrap().0, 1000); // Final bytes read
        assert_eq!(calls.last().unwrap().1, 1000); // Total size
    }
}
