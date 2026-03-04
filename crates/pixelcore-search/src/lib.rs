//! PixelCore Search Module
//!
//! Provides AI-enhanced search capabilities with full-text search,
//! vector search, and intelligent ranking.

pub mod engine;
pub mod indexer;
pub mod query;
pub mod ranking;
pub mod autocomplete;
pub mod cache;
pub mod error;

pub use engine::{SearchEngine, SearchEngineConfig};
pub use query::{SearchQuery, SearchResponse, SearchResult};
pub use error::{SearchError, Result};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Verify that all public exports are accessible
        let _config: Option<SearchEngineConfig> = None;
        let _query: Option<SearchQuery> = None;
        let _response: Option<SearchResponse> = None;
    }
}
