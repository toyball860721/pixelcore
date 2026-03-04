use crate::error::{AiError, Result};
use crate::recommendation::{RecommendationRequest, RecommendedItem, UserInteraction};
use std::collections::HashMap;
use uuid::Uuid;

/// Content-based recommendation algorithm
pub struct ContentBased {
    /// Item features (simplified: item_id -> feature vector)
    item_features: HashMap<Uuid, Vec<f64>>,
    /// User preferences (user_id -> preferred features)
    user_preferences: HashMap<Uuid, Vec<f64>>,
    /// Is model trained
    is_trained: bool,
}

impl ContentBased {
    pub fn new() -> Self {
        Self {
            item_features: HashMap::new(),
            user_preferences: HashMap::new(),
            is_trained: false,
        }
    }

    /// Train the content-based model
    pub fn train(&mut self, interactions: &[UserInteraction]) -> Result<()> {
        self.user_preferences.clear();

        // Build user preference profiles from interactions
        let mut user_interactions: HashMap<Uuid, Vec<&UserInteraction>> = HashMap::new();
        for interaction in interactions {
            user_interactions
                .entry(interaction.user_id)
                .or_insert_with(Vec::new)
                .push(interaction);
        }

        // Compute user preferences based on their interactions
        for (user_id, user_ints) in user_interactions {
            let mut preferences = vec![0.0; 10]; // 10-dimensional feature space

            for interaction in user_ints {
                if let Some(features) = self.item_features.get(&interaction.item_id) {
                    let weight = interaction.rating.unwrap_or(1.0);
                    for (i, feature) in features.iter().enumerate() {
                        preferences[i] += feature * weight;
                    }
                }
            }

            // Normalize preferences
            let magnitude: f64 = preferences.iter().map(|x| x * x).sum::<f64>().sqrt();
            if magnitude > 0.0 {
                for pref in &mut preferences {
                    *pref /= magnitude;
                }
            }

            self.user_preferences.insert(user_id, preferences);
        }

        self.is_trained = true;
        Ok(())
    }

    /// Add item features (would typically come from item metadata)
    pub fn add_item_features(&mut self, item_id: Uuid, features: Vec<f64>) {
        self.item_features.insert(item_id, features);
    }

    /// Get recommendations for a user
    pub fn recommend(&self, request: &RecommendationRequest) -> Result<Vec<RecommendedItem>> {
        if !self.is_trained {
            return Err(AiError::ModelNotTrained(
                "Content-based model not trained".to_string(),
            ));
        }

        let user_prefs = self.user_preferences.get(&request.user_id);
        if user_prefs.is_none() {
            // New user, return popular items
            return Ok(Vec::new());
        }

        let user_prefs = user_prefs.unwrap();

        // Score all items based on similarity to user preferences
        let mut item_scores: Vec<(Uuid, f64)> = self
            .item_features
            .iter()
            .filter_map(|(item_id, features)| {
                // Skip excluded items
                if let Some(ref exclude) = request.exclude_items {
                    if exclude.contains(item_id) {
                        return None;
                    }
                }

                // Calculate cosine similarity
                let score = self.cosine_similarity(user_prefs, features);
                Some((*item_id, score))
            })
            .collect();

        // Sort by score
        item_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        item_scores.truncate(request.limit);

        // Convert to RecommendedItem
        let recommendations = item_scores
            .into_iter()
            .map(|(item_id, score)| RecommendedItem {
                item_id,
                score,
                reason: "Matches your preferences".to_string(),
            })
            .collect();

        Ok(recommendations)
    }

    /// Calculate cosine similarity between two vectors
    fn cosine_similarity(&self, vec1: &[f64], vec2: &[f64]) -> f64 {
        if vec1.len() != vec2.len() {
            return 0.0;
        }

        let dot_product: f64 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();
        let magnitude1: f64 = vec1.iter().map(|x| x * x).sum::<f64>().sqrt();
        let magnitude2: f64 = vec2.iter().map(|x| x * x).sum::<f64>().sqrt();

        if magnitude1 == 0.0 || magnitude2 == 0.0 {
            return 0.0;
        }

        dot_product / (magnitude1 * magnitude2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recommendation::InteractionType;

    #[test]
    fn test_content_based_train() {
        let mut cb = ContentBased::new();

        let user1 = Uuid::new_v4();
        let item1 = Uuid::new_v4();

        // Add item features
        cb.add_item_features(item1, vec![1.0, 0.5, 0.3, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);

        let interactions = vec![UserInteraction {
            user_id: user1,
            item_id: item1,
            interaction_type: InteractionType::Purchase,
            rating: Some(5.0),
            timestamp: 1234567890,
        }];

        let result = cb.train(&interactions);
        assert!(result.is_ok());
        assert!(cb.is_trained);
    }

    #[test]
    fn test_cosine_similarity() {
        let cb = ContentBased::new();

        let vec1 = vec![1.0, 0.0, 0.0];
        let vec2 = vec![1.0, 0.0, 0.0];

        let similarity = cb.cosine_similarity(&vec1, &vec2);
        assert!((similarity - 1.0).abs() < 0.001);
    }
}
