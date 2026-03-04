use crate::error::Result;
use crate::recommendation::RecommendationResponse;
use redis::{aio::ConnectionManager, AsyncCommands};
use uuid::Uuid;

/// Recommendation cache using Redis
pub struct RecommendationCache {
    client: ConnectionManager,
    ttl: u64, // Time to live in seconds
}

impl RecommendationCache {
    /// Create a new recommendation cache
    pub async fn new(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let connection = ConnectionManager::new(client).await?;

        Ok(Self {
            client: connection,
            ttl: 3600, // 1 hour default
        })
    }

    /// Get cached recommendations for a user
    pub async fn get(&self, user_id: &Uuid) -> Result<Option<RecommendationResponse>> {
        let key = format!("recommendation:{}", user_id);
        let mut conn = self.client.clone();

        let cached: Option<String> = conn.get(&key).await?;

        match cached {
            Some(json) => {
                let response: RecommendationResponse = serde_json::from_str(&json)?;
                Ok(Some(response))
            }
            None => Ok(None),
        }
    }

    /// Cache recommendations for a user
    pub async fn set(&self, user_id: &Uuid, response: &RecommendationResponse) -> Result<()> {
        let key = format!("recommendation:{}", user_id);
        let json = serde_json::to_string(response)?;
        let mut conn = self.client.clone();

        let _: () = conn.set_ex(&key, json, self.ttl).await?;

        Ok(())
    }

    /// Invalidate cache for a user
    pub async fn invalidate(&self, user_id: &Uuid) -> Result<()> {
        let key = format!("recommendation:{}", user_id);
        let mut conn = self.client.clone();

        let _: () = conn.del(&key).await?;

        Ok(())
    }

    /// Set TTL for cache entries
    pub fn set_ttl(&mut self, ttl: u64) {
        self.ttl = ttl;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recommendation::RecommendedItem;

    #[tokio::test]
    #[ignore] // Requires Redis to be running
    async fn test_cache_operations() {
        let cache = RecommendationCache::new("redis://127.0.0.1:6379")
            .await
            .unwrap();

        let user_id = Uuid::new_v4();
        let response = RecommendationResponse {
            user_id,
            items: vec![RecommendedItem {
                item_id: Uuid::new_v4(),
                score: 0.95,
                reason: "Test".to_string(),
            }],
            algorithm: "test".to_string(),
            confidence: 0.85,
        };

        // Set cache
        cache.set(&user_id, &response).await.unwrap();

        // Get cache
        let cached = cache.get(&user_id).await.unwrap();
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().user_id, user_id);

        // Invalidate cache
        cache.invalidate(&user_id).await.unwrap();

        // Verify invalidation
        let cached = cache.get(&user_id).await.unwrap();
        assert!(cached.is_none());
    }
}
