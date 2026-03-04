//! AI-Enhanced Search Demo
//!
//! This example demonstrates the AI-powered search engine with
//! full-text search, autocomplete, and intelligent ranking.

use pixelcore_search::{
    query::{Document, SearchQuery},
    SearchEngine, SearchEngineConfig,
};
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔍 PixelCore AI-Enhanced Search Demo\n");

    // Initialize search engine
    println!("Initializing search engine...");
    let config = SearchEngineConfig {
        index_path: std::path::PathBuf::from("./data/search_index"),
        redis_url: std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
        cache_ttl: 300, // 5 minutes
        enable_autocomplete: true,
    };

    let engine = match SearchEngine::new(config).await {
        Ok(e) => e,
        Err(e) => {
            eprintln!("⚠️  Warning: Could not initialize search engine: {}", e);
            eprintln!("   Make sure Redis is running: docker run -d -p 6379:6379 redis\n");
            return Ok(());
        }
    };

    println!("✅ Search engine initialized!\n");

    // Create sample documents
    println!("Indexing sample documents...");

    let documents = vec![
        Document {
            id: Uuid::new_v4(),
            title: "Introduction to Rust Programming".to_string(),
            content: "Rust is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety.".to_string(),
            doc_type: "article".to_string(),
            tags: vec!["rust".to_string(), "programming".to_string()],
            metadata: serde_json::json!({"author": "John Doe"}),
            timestamp: 1234567890,
        },
        Document {
            id: Uuid::new_v4(),
            title: "Advanced Rust Patterns".to_string(),
            content: "Learn advanced Rust patterns including ownership, borrowing, lifetimes, and trait objects.".to_string(),
            doc_type: "article".to_string(),
            tags: vec!["rust".to_string(), "advanced".to_string()],
            metadata: serde_json::json!({"author": "Jane Smith"}),
            timestamp: 1234567891,
        },
        Document {
            id: Uuid::new_v4(),
            title: "Building Web Applications with Rust".to_string(),
            content: "Discover how to build fast and secure web applications using Rust and popular frameworks like Actix and Rocket.".to_string(),
            doc_type: "tutorial".to_string(),
            tags: vec!["rust".to_string(), "web".to_string()],
            metadata: serde_json::json!({"author": "Bob Johnson"}),
            timestamp: 1234567892,
        },
        Document {
            id: Uuid::new_v4(),
            title: "Python for Data Science".to_string(),
            content: "Python is the most popular language for data science, with libraries like NumPy, Pandas, and Scikit-learn.".to_string(),
            doc_type: "article".to_string(),
            tags: vec!["python".to_string(), "data-science".to_string()],
            metadata: serde_json::json!({"author": "Alice Brown"}),
            timestamp: 1234567893,
        },
        Document {
            id: Uuid::new_v4(),
            title: "Machine Learning with Rust".to_string(),
            content: "Explore machine learning in Rust using libraries like linfa, smartcore, and burn.".to_string(),
            doc_type: "tutorial".to_string(),
            tags: vec!["rust".to_string(), "machine-learning".to_string()],
            metadata: serde_json::json!({"author": "Charlie Wilson"}),
            timestamp: 1234567894,
        },
    ];

    engine.index_documents(documents).await?;
    println!("✅ Indexed {} documents\n", 5);

    // Perform searches
    println!("=== Search Examples ===\n");

    // Search 1: Basic search
    println!("1. Searching for 'rust'...");
    let query1 = SearchQuery {
        query: "rust".to_string(),
        limit: 10,
        ..Default::default()
    };

    match engine.search(query1).await {
        Ok(response) => {
            println!("   Found {} results in {}ms", response.total, response.query_time_ms);
            for (i, result) in response.results.iter().enumerate() {
                println!("   {}. {} (score: {:.3})", i + 1, result.title, result.score);
                println!("      {}", result.content);
            }
            println!();
        }
        Err(e) => {
            eprintln!("   ❌ Search error: {}", e);
        }
    }

    // Search 2: More specific search
    println!("2. Searching for 'web applications'...");
    let query2 = SearchQuery {
        query: "web applications".to_string(),
        limit: 5,
        ..Default::default()
    };

    match engine.search(query2).await {
        Ok(response) => {
            println!("   Found {} results in {}ms", response.total, response.query_time_ms);
            for (i, result) in response.results.iter().enumerate() {
                println!("   {}. {} (score: {:.3})", i + 1, result.title, result.score);
            }
            println!();
        }
        Err(e) => {
            eprintln!("   ❌ Search error: {}", e);
        }
    }

    // Search 3: Different topic
    println!("3. Searching for 'python data'...");
    let query3 = SearchQuery {
        query: "python data".to_string(),
        limit: 5,
        ..Default::default()
    };

    match engine.search(query3).await {
        Ok(response) => {
            println!("   Found {} results in {}ms", response.total, response.query_time_ms);
            for (i, result) in response.results.iter().enumerate() {
                println!("   {}. {} (score: {:.3})", i + 1, result.title, result.score);
            }
            println!();
        }
        Err(e) => {
            eprintln!("   ❌ Search error: {}", e);
        }
    }

    // Autocomplete demo
    println!("=== Autocomplete Examples ===\n");

    let prefixes = vec!["rus", "prog", "mach"];
    for prefix in prefixes {
        let suggestions = engine.autocomplete(prefix, 5);
        println!("Autocomplete for '{}': {:?}", prefix, suggestions);
    }
    println!();

    // Get index statistics
    println!("=== Index Statistics ===\n");
    match engine.stats().await {
        Ok(stats) => {
            println!("Total documents: {}", stats.total_documents);
            println!("Index size: {} bytes", stats.index_size_bytes);
        }
        Err(e) => {
            eprintln!("❌ Error getting stats: {}", e);
        }
    }

    println!("\n🎉 Demo completed successfully!");

    Ok(())
}
