use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Search query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Query text
    pub query: String,
    /// Number of results to return
    pub limit: usize,
    /// Offset for pagination
    pub offset: usize,
    /// Filters (field -> value)
    pub filters: Option<Vec<SearchFilter>>,
    /// Sort by field
    pub sort_by: Option<String>,
    /// Sort order
    pub sort_order: SortOrder,
    /// Enable fuzzy search
    pub fuzzy: bool,
    /// Highlight matches
    pub highlight: bool,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            query: String::new(),
            limit: 10,
            offset: 0,
            filters: None,
            sort_by: None,
            sort_order: SortOrder::Descending,
            fuzzy: true,
            highlight: true,
        }
    }
}

/// Search filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilter {
    pub field: String,
    pub value: String,
    pub operator: FilterOperator,
}

/// Filter operator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterOperator {
    Equals,
    Contains,
    GreaterThan,
    LessThan,
    Range(String, String),
}

/// Sort order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,
    Descending,
}

/// Search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    /// Search results
    pub results: Vec<SearchResult>,
    /// Total number of results
    pub total: usize,
    /// Query time in milliseconds
    pub query_time_ms: u64,
    /// Suggestions for query correction
    pub suggestions: Vec<String>,
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Document ID
    pub id: Uuid,
    /// Document title
    pub title: String,
    /// Document content (snippet)
    pub content: String,
    /// Relevance score
    pub score: f32,
    /// Highlighted snippets
    pub highlights: Vec<String>,
    /// Metadata
    pub metadata: serde_json::Value,
}

/// Document to be indexed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Document ID
    pub id: Uuid,
    /// Document title
    pub title: String,
    /// Document content
    pub content: String,
    /// Document type
    pub doc_type: String,
    /// Tags
    pub tags: Vec<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Timestamp
    pub timestamp: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_query_default() {
        let query = SearchQuery::default();
        assert_eq!(query.limit, 10);
        assert_eq!(query.offset, 0);
        assert!(query.fuzzy);
        assert!(query.highlight);
    }

    #[test]
    fn test_document_creation() {
        let doc = Document {
            id: Uuid::new_v4(),
            title: "Test Document".to_string(),
            content: "This is a test document".to_string(),
            doc_type: "text".to_string(),
            tags: vec!["test".to_string()],
            metadata: serde_json::json!({}),
            timestamp: 1234567890,
        };

        assert_eq!(doc.title, "Test Document");
        assert_eq!(doc.tags.len(), 1);
    }
}
