use crate::query::{SearchQuery, SearchResult};

/// Search result ranker
pub struct Ranker {
    // Ranking weights
    title_weight: f32,
    content_weight: f32,
    recency_weight: f32,
}

impl Ranker {
    pub fn new() -> Self {
        Self {
            title_weight: 2.0,
            content_weight: 1.0,
            recency_weight: 0.5,
        }
    }

    /// Rank search results
    pub fn rank(&self, mut results: Vec<SearchResult>, query: &SearchQuery) -> Vec<SearchResult> {
        // Apply custom ranking
        for result in &mut results {
            let mut adjusted_score = result.score;

            // Boost title matches
            if result.title.to_lowercase().contains(&query.query.to_lowercase()) {
                adjusted_score *= self.title_weight;
            }

            // Boost exact matches
            if result.title.to_lowercase() == query.query.to_lowercase() {
                adjusted_score *= 3.0;
            }

            result.score = adjusted_score;
        }

        // Sort by adjusted score
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        results
    }

    /// Set ranking weights
    pub fn set_weights(&mut self, title: f32, content: f32, recency: f32) {
        self.title_weight = title;
        self.content_weight = content;
        self.recency_weight = recency;
    }
}

impl Default for Ranker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_ranker_creation() {
        let ranker = Ranker::new();
        assert_eq!(ranker.title_weight, 2.0);
        assert_eq!(ranker.content_weight, 1.0);
    }

    #[test]
    fn test_ranking() {
        let ranker = Ranker::new();

        let mut results = vec![
            SearchResult {
                id: Uuid::new_v4(),
                title: "Test Document".to_string(),
                content: "Content".to_string(),
                score: 1.0,
                highlights: vec![],
                metadata: serde_json::json!({}),
            },
            SearchResult {
                id: Uuid::new_v4(),
                title: "Another Document".to_string(),
                content: "Content".to_string(),
                score: 2.0,
                highlights: vec![],
                metadata: serde_json::json!({}),
            },
        ];

        let query = SearchQuery {
            query: "test".to_string(),
            ..Default::default()
        };

        let ranked = ranker.rank(results, &query);
        assert_eq!(ranked.len(), 2);
    }
}
