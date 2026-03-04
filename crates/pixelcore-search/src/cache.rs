use crate::error::Result;
use crate::query::{SearchQuery, SearchResponse};
use redis::{aio::ConnectionManager, AsyncCommands};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Search result cache using Redis
pub struct SearchCache {
    client: ConnectionManager,
    ttl: u64, // Time to live in seconds
}

impl SearchCache {
    /// Create a new search cache
    pub async fn new(redis_url: &str, ttl: u64) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let connection = ConnectionManager::new(client).await?;

        Ok(Self {
            client: connection,
            ttl,
        })
    }

    /// Get cached search results
    pub async fn get(&self, query: &SearchQuery) -> Result<Option<SearchResponse>> {
        let key = self.query_key(query);
        let mut conn = self.client.clone();

        let cached: Option<String> = conn.get(&key).await?;

        match cached {
            Some(json) => {
                let response: SearchResponse = serde_json::from_str(&json)?;
                Ok(Some(response))
            }
            None => Ok(None),
        }
    }

    /// Cache search results
    pub async fn set(&self, query: &SearchQuery, response: &SearchResponse) -> Result<()> {
        let key = self.query_key(query);
        let json = serde_json::to_string(response)?;
        let mut conn = self.client.clone();

        let _: () = conn.set_ex(&key, json, self.ttl).await?;

        Ok(())
    }

    /// Invalidate cache
    pub async fn invalidate(&self, query: &SearchQuery) -> Result<()> {
        let key = self.query_key(query);
        let mut conn = self.client.clone();

        let _: () = conn.del(&key).await?;

        Ok(())
    }

    /// Clear all cache
    pub async fn clear_all(&self) -> Result<()> {
        let mut conn = self.client.clone();
        let _: () = conn.del("search:*").await?;
        Ok(())
    }

    /// Generate cache key from query
    fn query_key(&self, query: &SearchQuery) -> String {
        let mut hasher = DefaultHasher::new();
        query.query.hash(&mut hasher);
        query.limit.hash(&mut hasher);
        query.offset.hash(&mut hasher);

        format!("search:{}", hasher.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::SearchResult;
    use uuid::Uuid;

    #[tokio::test]
    #[ignore] // Requires Redis to be running
    async fn test_cache_operations() {
        let cache = SearchCache::new("redis://127.0.0.1:6379", 300)
            .await
            .unwrap();

        let query = SearchQuery {
            query: "test".to_string(),
            ..Default::default()
        };

        let response = SearchResponse {
            results: vec![SearchResult {
                id: Uuid::new_v4(),
                title: "Test".to_string(),
                content: "Content".to_string(),
                score: 1.0,
                highlights: vec![],
                metadata: serde_json::json!({}),
            }],
            total: 1,
            query_time_ms: 10,
            suggestions: vec![],
        };

        // Set cache
        cache.set(&query, &response).await.unwrap();

        // Get cache
        let cached = cache.get(&query).await.unwrap();
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().total, 1);

        // Invalidate cache
        cache.invalidate(&query).await.unwrap();

        // Verify invalidation
        let cached = cache.get(&query).await.unwrap();
        assert!(cached.is_none());
    }
}
