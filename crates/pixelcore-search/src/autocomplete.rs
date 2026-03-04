use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use unicode_segmentation::UnicodeSegmentation;

/// Autocomplete engine using Trie
pub struct AutoComplete {
    trie: Arc<RwLock<TrieNode>>,
}

impl AutoComplete {
    pub fn new() -> Self {
        Self {
            trie: Arc::new(RwLock::new(TrieNode::new())),
        }
    }

    /// Add a phrase to the autocomplete index
    pub fn add_phrase(&self, phrase: &str) {
        let words: Vec<&str> = phrase.unicode_words().collect();

        for word in words {
            let word_lower = word.to_lowercase();
            if word_lower.len() >= 2 {
                // Only index words with 2+ characters
                let mut trie = self.trie.write().unwrap();
                trie.insert(&word_lower);
            }
        }
    }

    /// Get autocomplete suggestions
    pub fn suggest(&self, prefix: &str, limit: usize) -> Vec<String> {
        let prefix_lower = prefix.to_lowercase();
        let trie = self.trie.read().unwrap();
        trie.search(&prefix_lower, limit)
    }

    /// Clear all autocomplete data
    pub fn clear(&self) {
        let mut trie = self.trie.write().unwrap();
        *trie = TrieNode::new();
    }
}

impl Default for AutoComplete {
    fn default() -> Self {
        Self::new()
    }
}

/// Trie node for autocomplete
struct TrieNode {
    children: HashMap<char, TrieNode>,
    is_end: bool,
    frequency: usize,
}

impl TrieNode {
    fn new() -> Self {
        Self {
            children: HashMap::new(),
            is_end: false,
            frequency: 0,
        }
    }

    /// Insert a word into the trie
    fn insert(&mut self, word: &str) {
        let mut current = self;

        for ch in word.chars() {
            current = current.children.entry(ch).or_insert_with(TrieNode::new);
        }

        current.is_end = true;
        current.frequency += 1;
    }

    /// Search for words with given prefix
    fn search(&self, prefix: &str, limit: usize) -> Vec<String> {
        // Navigate to the prefix node
        let mut current = self;
        for ch in prefix.chars() {
            match current.children.get(&ch) {
                Some(node) => current = node,
                None => return Vec::new(), // Prefix not found
            }
        }

        // Collect all words from this node
        let mut results = Vec::new();
        self.collect_words(current, prefix.to_string(), &mut results, limit);

        // Sort by frequency
        results.sort_by(|a, b| b.1.cmp(&a.1));

        // Return only the words (without frequency)
        results.into_iter().take(limit).map(|(word, _)| word).collect()
    }

    /// Recursively collect words from a node
    fn collect_words(
        &self,
        node: &TrieNode,
        prefix: String,
        results: &mut Vec<(String, usize)>,
        limit: usize,
    ) {
        if results.len() >= limit * 2 {
            // Collect more than needed for better sorting
            return;
        }

        if node.is_end {
            results.push((prefix.clone(), node.frequency));
        }

        for (ch, child) in &node.children {
            let mut new_prefix = prefix.clone();
            new_prefix.push(*ch);
            self.collect_words(child, new_prefix, results, limit);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autocomplete_creation() {
        let ac = AutoComplete::new();
        let suggestions = ac.suggest("test", 5);
        assert_eq!(suggestions.len(), 0);
    }

    #[test]
    fn test_add_and_suggest() {
        let ac = AutoComplete::new();

        ac.add_phrase("hello world");
        ac.add_phrase("hello there");
        ac.add_phrase("help me");

        let suggestions = ac.suggest("hel", 5);
        assert!(!suggestions.is_empty());
        assert!(suggestions.contains(&"hello".to_string()) || suggestions.contains(&"help".to_string()));
    }

    #[test]
    fn test_frequency_ranking() {
        let ac = AutoComplete::new();

        // Add "test" multiple times
        ac.add_phrase("test");
        ac.add_phrase("test");
        ac.add_phrase("test");
        ac.add_phrase("testing");

        let suggestions = ac.suggest("tes", 5);
        assert!(!suggestions.is_empty());
        // "test" should appear before "testing" due to higher frequency
        if suggestions.len() >= 2 {
            assert_eq!(suggestions[0], "test");
        }
    }

    #[test]
    fn test_clear() {
        let ac = AutoComplete::new();

        ac.add_phrase("hello world");
        let suggestions_before = ac.suggest("hel", 5);
        assert!(!suggestions_before.is_empty());

        ac.clear();
        let suggestions_after = ac.suggest("hel", 5);
        assert_eq!(suggestions_after.len(), 0);
    }
}
