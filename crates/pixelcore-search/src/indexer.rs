use crate::error::{Result, SearchError};
use crate::query::{Document, SearchQuery, SearchResult};
use crate::engine::IndexStats;
use std::path::Path;
use tantivy::schema::*;
use tantivy::{Index, TantivyDocument};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::OwnedValue;
use uuid::Uuid;

/// Document indexer using Tantivy
pub struct Indexer {
    index: Index,
    schema: Schema,
    id_field: Field,
    title_field: Field,
    content_field: Field,
    doc_type_field: Field,
    tags_field: Field,
    timestamp_field: Field,
}

impl Indexer {
    /// Create a new indexer
    pub fn new(index_path: &Path) -> Result<Self> {
        // Build schema
        let mut schema_builder = Schema::builder();

        let id_field = schema_builder.add_text_field("id", STRING | STORED);
        let title_field = schema_builder.add_text_field("title", TEXT | STORED);
        let content_field = schema_builder.add_text_field("content", TEXT | STORED);
        let doc_type_field = schema_builder.add_text_field("doc_type", STRING | STORED);
        let tags_field = schema_builder.add_text_field("tags", TEXT | STORED);
        let timestamp_field = schema_builder.add_i64_field("timestamp", INDEXED | STORED);

        let schema = schema_builder.build();

        // Create or open index
        let meta_path = index_path.join("meta.json");
        let index = if meta_path.exists() {
            Index::open_in_dir(index_path)?
        } else {
            std::fs::create_dir_all(index_path)?;
            Index::create_in_dir(index_path, schema.clone())?
        };

        Ok(Self {
            index,
            schema,
            id_field,
            title_field,
            content_field,
            doc_type_field,
            tags_field,
            timestamp_field,
        })
    }

    /// Add a document to the index
    pub fn add_document(&mut self, document: Document) -> Result<()> {
        let mut index_writer = self.index.writer::<TantivyDocument>(50_000_000)?;

        let mut doc = TantivyDocument::new();
        doc.add_text(self.id_field, document.id.to_string());
        doc.add_text(self.title_field, &document.title);
        doc.add_text(self.content_field, &document.content);
        doc.add_text(self.doc_type_field, &document.doc_type);
        doc.add_text(self.tags_field, document.tags.join(" "));
        doc.add_i64(self.timestamp_field, document.timestamp);

        index_writer.add_document(doc)?;
        index_writer.commit()?;

        Ok(())
    }

    /// Search documents
    pub fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let reader = self.index.reader()?;
        let searcher = reader.searcher();

        // Parse query
        let query_parser = QueryParser::for_index(
            &self.index,
            vec![self.title_field, self.content_field, self.tags_field],
        );

        let parsed_query = query_parser
            .parse_query(&query.query)
            .map_err(|e| SearchError::QueryParseError(e.to_string()))?;

        // Execute search
        let top_docs = searcher.search(
            &parsed_query,
            &TopDocs::with_limit(query.limit + query.offset),
        )?;

        // Convert results
        let mut results = Vec::new();
        for (_score, doc_address) in top_docs.into_iter().skip(query.offset) {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;

            let id_str = retrieved_doc
                .get_first(self.id_field)
                .and_then(|v: &OwnedValue| v.as_str())
                .ok_or_else(|| SearchError::InternalError("Missing id field".to_string()))?;

            let id = Uuid::parse_str(id_str)
                .map_err(|e| SearchError::InternalError(format!("Invalid UUID: {}", e)))?;

            let title = retrieved_doc
                .get_first(self.title_field)
                .and_then(|v: &OwnedValue| v.as_str())
                .unwrap_or("")
                .to_string();

            let content = retrieved_doc
                .get_first(self.content_field)
                .and_then(|v: &OwnedValue| v.as_str())
                .unwrap_or("")
                .to_string();

            // Create snippet (first 200 chars)
            let snippet = if content.len() > 200 {
                format!("{}...", &content[..200])
            } else {
                content.clone()
            };

            results.push(SearchResult {
                id,
                title,
                content: snippet,
                score: _score,
                highlights: vec![],
                metadata: serde_json::json!({}),
            });
        }

        Ok(results)
    }

    /// Delete a document
    pub fn delete_document(&mut self, id: Uuid) -> Result<()> {
        let mut index_writer = self.index.writer::<TantivyDocument>(50_000_000)?;
        let term = tantivy::Term::from_field_text(self.id_field, &id.to_string());
        index_writer.delete_term(term);
        index_writer.commit()?;
        Ok(())
    }

    /// Commit changes
    pub fn commit(&mut self) -> Result<()> {
        let mut index_writer = self.index.writer::<TantivyDocument>(50_000_000)?;
        index_writer.commit()?;
        Ok(())
    }

    /// Get index statistics
    pub fn stats(&self) -> Result<IndexStats> {
        let reader = self.index.reader()?;
        let searcher = reader.searcher();

        let total_documents = searcher.num_docs();

        // Estimate index size
        let index_size_bytes = 0; // TODO: Calculate actual size

        Ok(IndexStats {
            total_documents,
            index_size_bytes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_indexer_creation() {
        let temp_dir = TempDir::new().unwrap();
        let indexer = Indexer::new(temp_dir.path());
        assert!(indexer.is_ok());
    }

    #[test]
    fn test_add_and_search_document() {
        let temp_dir = TempDir::new().unwrap();
        let mut indexer = Indexer::new(temp_dir.path()).unwrap();

        let doc = Document {
            id: Uuid::new_v4(),
            title: "Test Document".to_string(),
            content: "This is a test document with some content".to_string(),
            doc_type: "text".to_string(),
            tags: vec!["test".to_string()],
            metadata: serde_json::json!({}),
            timestamp: 1234567890,
        };

        let result = indexer.add_document(doc);
        assert!(result.is_ok());

        // Search for the document
        let query = SearchQuery {
            query: "test".to_string(),
            limit: 10,
            offset: 0,
            filters: None,
            sort_by: None,
            sort_order: crate::query::SortOrder::Descending,
            fuzzy: false,
            highlight: false,
        };

        let results = indexer.search(&query).unwrap();
        assert!(!results.is_empty());
    }
}
