use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Recommendation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationRequest {
    /// User ID
    pub user_id: Uuid,
    /// Number of recommendations to return
    pub limit: usize,
    /// Item type filter (optional)
    pub item_type: Option<String>,
    /// Exclude items (optional)
    pub exclude_items: Option<Vec<Uuid>>,
}

/// Recommendation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationResponse {
    /// User ID
    pub user_id: Uuid,
    /// Recommended items
    pub items: Vec<RecommendedItem>,
    /// Algorithm used
    pub algorithm: String,
    /// Confidence score
    pub confidence: f64,
}

/// Recommended item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedItem {
    /// Item ID
    pub item_id: Uuid,
    /// Recommendation score
    pub score: f64,
    /// Reason for recommendation
    pub reason: String,
}

/// User interaction data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInteraction {
    pub user_id: Uuid,
    pub item_id: Uuid,
    pub interaction_type: InteractionType,
    pub rating: Option<f64>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractionType {
    View,
    Click,
    Purchase,
    Like,
    Share,
    Rating,
}

use crate::error::Result;
use crate::collaborative_filtering::CollaborativeFiltering;
use crate::content_based::ContentBased;
use crate::cache::RecommendationCache;

/// Recommendation engine
pub struct RecommendationEngine {
    collaborative: CollaborativeFiltering,
    content_based: ContentBased,
    cache: RecommendationCache,
}

impl RecommendationEngine {
    /// Create a new recommendation engine
    pub async fn new(redis_url: &str) -> Result<Self> {
        Ok(Self {
            collaborative: CollaborativeFiltering::new(),
            content_based: ContentBased::new(),
            cache: RecommendationCache::new(redis_url).await?,
        })
    }

    /// Get recommendations for a user
    pub async fn recommend(&self, request: RecommendationRequest) -> Result<RecommendationResponse> {
        // Check cache first
        if let Some(cached) = self.cache.get(&request.user_id).await? {
            return Ok(cached);
        }

        // Try collaborative filtering first
        let mut items = self.collaborative.recommend(&request)?;

        // If not enough items, supplement with content-based
        if items.len() < request.limit {
            let content_items = self.content_based.recommend(&request)?;
            items.extend(content_items);
        }

        // Limit to requested number
        items.truncate(request.limit);

        let response = RecommendationResponse {
            user_id: request.user_id,
            items,
            algorithm: "hybrid".to_string(),
            confidence: 0.85,
        };

        // Cache the result
        self.cache.set(&request.user_id, &response).await?;

        Ok(response)
    }

    /// Train the recommendation models
    pub async fn train(&mut self, interactions: Vec<UserInteraction>) -> Result<()> {
        self.collaborative.train(&interactions)?;
        self.content_based.train(&interactions)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_recommendation_request() {
        let request = RecommendationRequest {
            user_id: Uuid::new_v4(),
            limit: 10,
            item_type: Some("image".to_string()),
            exclude_items: None,
        };

        assert_eq!(request.limit, 10);
    }

    #[test]
    fn test_interaction_type() {
        let interaction = UserInteraction {
            user_id: Uuid::new_v4(),
            item_id: Uuid::new_v4(),
            interaction_type: InteractionType::Purchase,
            rating: Some(5.0),
            timestamp: 1234567890,
        };

        assert!(matches!(interaction.interaction_type, InteractionType::Purchase));
    }
}
