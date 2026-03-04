use crate::error::{AiError, Result};
use crate::recommendation::{RecommendationRequest, RecommendedItem, UserInteraction};
use std::collections::HashMap;
use uuid::Uuid;

/// Collaborative filtering recommendation algorithm
pub struct CollaborativeFiltering {
    /// User-item interaction matrix
    user_item_matrix: HashMap<Uuid, HashMap<Uuid, f64>>,
    /// User similarity cache
    user_similarity: HashMap<(Uuid, Uuid), f64>,
    /// Is model trained
    is_trained: bool,
}

impl CollaborativeFiltering {
    pub fn new() -> Self {
        Self {
            user_item_matrix: HashMap::new(),
            user_similarity: HashMap::new(),
            is_trained: false,
        }
    }

    /// Train the collaborative filtering model
    pub fn train(&mut self, interactions: &[UserInteraction]) -> Result<()> {
        self.user_item_matrix.clear();
        self.user_similarity.clear();

        // Build user-item matrix
        for interaction in interactions {
            let rating = interaction.rating.unwrap_or(1.0);
            self.user_item_matrix
                .entry(interaction.user_id)
                .or_insert_with(HashMap::new)
                .insert(interaction.item_id, rating);
        }

        // Compute user similarities
        self.compute_user_similarities()?;

        self.is_trained = true;
        Ok(())
    }

    /// Compute cosine similarity between all users
    fn compute_user_similarities(&mut self) -> Result<()> {
        let users: Vec<Uuid> = self.user_item_matrix.keys().copied().collect();

        for i in 0..users.len() {
            for j in (i + 1)..users.len() {
                let user1 = users[i];
                let user2 = users[j];

                if let Some(similarity) = self.cosine_similarity(user1, user2) {
                    self.user_similarity.insert((user1, user2), similarity);
                    self.user_similarity.insert((user2, user1), similarity);
                }
            }
        }

        Ok(())
    }

    /// Calculate cosine similarity between two users
    fn cosine_similarity(&self, user1: Uuid, user2: Uuid) -> Option<f64> {
        let items1 = self.user_item_matrix.get(&user1)?;
        let items2 = self.user_item_matrix.get(&user2)?;

        // Find common items
        let common_items: Vec<&Uuid> = items1
            .keys()
            .filter(|item| items2.contains_key(item))
            .collect();

        if common_items.is_empty() {
            return None;
        }

        // Calculate dot product and magnitudes
        let mut dot_product = 0.0;
        let mut magnitude1 = 0.0;
        let mut magnitude2 = 0.0;

        for item in common_items {
            let rating1 = items1[item];
            let rating2 = items2[item];

            dot_product += rating1 * rating2;
            magnitude1 += rating1 * rating1;
            magnitude2 += rating2 * rating2;
        }

        if magnitude1 == 0.0 || magnitude2 == 0.0 {
            return None;
        }

        Some(dot_product / (magnitude1.sqrt() * magnitude2.sqrt()))
    }

    /// Get recommendations for a user
    pub fn recommend(&self, request: &RecommendationRequest) -> Result<Vec<RecommendedItem>> {
        if !self.is_trained {
            return Err(AiError::ModelNotTrained(
                "Collaborative filtering model not trained".to_string(),
            ));
        }

        let user_items = self.user_item_matrix.get(&request.user_id);
        if user_items.is_none() {
            // New user, return empty recommendations
            return Ok(Vec::new());
        }

        let user_items = user_items.unwrap();

        // Find similar users
        let mut similar_users: Vec<(Uuid, f64)> = self
            .user_similarity
            .iter()
            .filter(|((u1, _), _)| *u1 == request.user_id)
            .map(|((_, u2), sim)| (*u2, *sim))
            .collect();

        similar_users.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        similar_users.truncate(10); // Top 10 similar users

        // Aggregate recommendations from similar users
        let mut item_scores: HashMap<Uuid, f64> = HashMap::new();

        for (similar_user, similarity) in similar_users {
            if let Some(similar_user_items) = self.user_item_matrix.get(&similar_user) {
                for (item_id, rating) in similar_user_items {
                    // Skip items the user has already interacted with
                    if user_items.contains_key(item_id) {
                        continue;
                    }

                    // Skip excluded items
                    if let Some(ref exclude) = request.exclude_items {
                        if exclude.contains(item_id) {
                            continue;
                        }
                    }

                    *item_scores.entry(*item_id).or_insert(0.0) += rating * similarity;
                }
            }
        }

        // Convert to RecommendedItem and sort by score
        let mut recommendations: Vec<RecommendedItem> = item_scores
            .into_iter()
            .map(|(item_id, score)| RecommendedItem {
                item_id,
                score,
                reason: "Similar users also liked this".to_string(),
            })
            .collect();

        recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        recommendations.truncate(request.limit);

        Ok(recommendations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recommendation::InteractionType;

    #[test]
    fn test_collaborative_filtering_train() {
        let mut cf = CollaborativeFiltering::new();

        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();
        let item1 = Uuid::new_v4();
        let item2 = Uuid::new_v4();

        let interactions = vec![
            UserInteraction {
                user_id: user1,
                item_id: item1,
                interaction_type: InteractionType::Purchase,
                rating: Some(5.0),
                timestamp: 1234567890,
            },
            UserInteraction {
                user_id: user2,
                item_id: item1,
                interaction_type: InteractionType::Purchase,
                rating: Some(4.0),
                timestamp: 1234567891,
            },
            UserInteraction {
                user_id: user2,
                item_id: item2,
                interaction_type: InteractionType::Purchase,
                rating: Some(5.0),
                timestamp: 1234567892,
            },
        ];

        let result = cf.train(&interactions);
        assert!(result.is_ok());
        assert!(cf.is_trained);
    }

    #[test]
    fn test_cosine_similarity() {
        let mut cf = CollaborativeFiltering::new();

        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();
        let item1 = Uuid::new_v4();

        cf.user_item_matrix.insert(user1, {
            let mut map = HashMap::new();
            map.insert(item1, 5.0);
            map
        });

        cf.user_item_matrix.insert(user2, {
            let mut map = HashMap::new();
            map.insert(item1, 4.0);
            map
        });

        let similarity = cf.cosine_similarity(user1, user2);
        assert!(similarity.is_some());
        assert!(similarity.unwrap() > 0.9); // Should be close to 1.0
    }
}
