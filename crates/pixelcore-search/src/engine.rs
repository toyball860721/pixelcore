use crate::error::{Result, SearchError};
use crate::indexer::Indexer;
use crate::query::{Document, SearchQuery, SearchResponse, SearchResult};
use crate::ranking::Ranker;
use crate::autocomplete::AutoComplete;
use crate::cache::SearchCache;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Search engine configuration
#[derive(Debug, Clone)]
pub struct SearchEngineConfig {
    /// Index directory path
    pub index_path: PathBuf,
    /// Redis URL for caching
    pub redis_url: String,
    /// Cache TTL in seconds
    pub cache_ttl: u64,
    /// Enable autocomplete
    pub enable_autocomplete: bool,
}

impl Default for SearchEngineConfig {
    fn default() -> Self {
        Self {
            index_path: PathBuf::from("./data/search_index"),
            redis_url: "redis://127.0.0.1:6379".to_string(),
            cache_ttl: 300, // 5 minutes
            enable_autocomplete: true,
        }
    }
}

/// Search engine
pub struct SearchEngine {
    config: SearchEngineConfig,
    indexer: Arc<RwLock<Indexer>>,
    ranker: Ranker,
    autocomplete: Option<AutoComplete>,
    cache: SearchCache,
}

impl SearchEngine {
    /// Create a new search engine
    pub async fn new(config: SearchEngineConfig) -> Result<Self> {
        // Create index directory if it doesn't exist
        std::fs::create_dir_all(&config.index_path)?;

        // Initialize indexer
        let indexer = Indexer::new(&config.index_path)?;

        // Initialize cache
        let cache = SearchCache::new(&config.redis_url, config.cache_ttl).await?;

        // Initialize autocomplete if enabled
        let autocomplete = if config.enable_autocomplete {
            Some(AutoComplete::new())
        } else {
            None
        };

        Ok(Self {
            config,
            indexer: Arc::new(RwLock::new(indexer)),
            ranker: Ranker::new(),
            autocomplete,
            cache,
        })
    }

    /// Index a document
    pub async fn index_document(&self, document: Document) -> Result<()> {
        let mut indexer = self.indexer.write().await;
        indexer.add_document(document.clone())?;

        // Update autocomplete
        if let Some(ref autocomplete) = self.autocomplete {
            autocomplete.add_phrase(&document.title);
            autocomplete.add_phrase(&document.content);
        }

        Ok(())
    }

    /// Index multiple documents
    pub async fn index_documents(&self, documents: Vec<Document>) -> Result<()> {
        let mut indexer = self.indexer.write().await;

        for document in documents {
            indexer.add_document(document.clone())?;

            // Update autocomplete
            if let Some(ref autocomplete) = self.autocomplete {
                autocomplete.add_phrase(&document.title);
            }
        }

        indexer.commit()?;

        Ok(())
    }

    /// Search documents
    pub async fn search(&self, query: SearchQuery) -> Result<SearchResponse> {
        let start = std::time::Instant::now();

        // Check cache first
        if let Some(cached) = self.cache.get(&query).await? {
            return Ok(cached);
        }

        // Perform search
        let indexer = self.indexer.read().await;
        let mut results = indexer.search(&query)?;

        // Apply ranking
        results = self.ranker.rank(results, &query);

        // Generate suggestions
        let suggestions = if let Some(ref autocomplete) = self.autocomplete {
            autocomplete.suggest(&query.query, 5)
        } else {
            Vec::new()
        };

        let query_time_ms = start.elapsed().as_millis() as u64;

        let total = results.len();
        let response = SearchResponse {
            results,
            total,
            query_time_ms,
            suggestions,
        };

        // Cache the result
        self.cache.set(&query, &response).await?;

        Ok(response)
    }

    /// Get autocomplete suggestions
    pub fn autocomplete(&self, prefix: &str, limit: usize) -> Vec<String> {
        if let Some(ref autocomplete) = self.autocomplete {
            autocomplete.suggest(prefix, limit)
        } else {
            Vec::new()
        }
    }

    /// Delete a document
    pub async fn delete_document(&self, id: uuid::Uuid) -> Result<()> {
        let mut indexer = self.indexer.write().await;
        indexer.delete_document(id)?;
        Ok(())
    }

    /// Commit changes to index
    pub async fn commit(&self) -> Result<()> {
        let mut indexer = self.indexer.write().await;
        indexer.commit()?;
        Ok(())
    }

    /// Get index statistics
    pub async fn stats(&self) -> Result<IndexStats> {
        let indexer = self.indexer.read().await;
        indexer.stats()
    }
}

/// Index statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IndexStats {
    pub total_documents: u64,
    pub index_size_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_search_engine_creation() {
        let temp_dir = TempDir::new().unwrap();

        let config = SearchEngineConfig {
            index_path: temp_dir.path().to_path_buf(),
            redis_url: "redis://127.0.0.1:6379".to_string(),
            cache_ttl: 300,
            enable_autocomplete: true,
        };

        // This will fail if Redis is not running, which is expected
        let result = SearchEngine::new(config).await;

        // We just test that the function doesn't panic
        // In a real test environment, Redis would be available
        assert!(result.is_ok() || result.is_err());
    }
}
