//! PixelCore AI Module
//!
//! Provides AI-powered recommendation system with collaborative filtering
//! and content-based recommendation algorithms.

pub mod recommendation;
pub mod collaborative_filtering;
pub mod content_based;
pub mod cache;
pub mod error;

pub use recommendation::{RecommendationEngine, RecommendationRequest, RecommendationResponse};
pub use error::{AiError, Result};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Verify that all public exports are accessible
        let _engine: Option<RecommendationEngine> = None;
        let _request: Option<RecommendationRequest> = None;
        let _response: Option<RecommendationResponse> = None;
    }
}
