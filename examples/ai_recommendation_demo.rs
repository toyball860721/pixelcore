//! AI Recommendation System Demo
//!
//! This example demonstrates the AI-powered recommendation system
//! with collaborative filtering and content-based algorithms.

use pixelcore_ai::{
    recommendation::{InteractionType, RecommendationRequest, UserInteraction},
    RecommendationEngine,
};
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🤖 PixelCore AI Recommendation System Demo\n");

    // Initialize recommendation engine
    println!("Initializing recommendation engine...");
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    let mut engine = match RecommendationEngine::new(&redis_url).await {
        Ok(e) => e,
        Err(e) => {
            eprintln!("⚠️  Warning: Could not connect to Redis: {}", e);
            eprintln!("   Recommendation caching will not be available.");
            eprintln!("   To enable caching, start Redis: docker run -d -p 6379:6379 redis\n");
            return Ok(());
        }
    };

    // Create sample users and items
    let user1 = Uuid::new_v4();
    let user2 = Uuid::new_v4();
    let user3 = Uuid::new_v4();

    let item1 = Uuid::new_v4();
    let item2 = Uuid::new_v4();
    let item3 = Uuid::new_v4();
    let item4 = Uuid::new_v4();
    let item5 = Uuid::new_v4();

    println!("Created sample users and items:");
    println!("  Users: {} {} {}", user1, user2, user3);
    println!("  Items: {} {} {} {} {}\n", item1, item2, item3, item4, item5);

    // Create sample interactions
    let interactions = vec![
        // User 1 likes items 1, 2, 3
        UserInteraction {
            user_id: user1,
            item_id: item1,
            interaction_type: InteractionType::Purchase,
            rating: Some(5.0),
            timestamp: 1234567890,
        },
        UserInteraction {
            user_id: user1,
            item_id: item2,
            interaction_type: InteractionType::Like,
            rating: Some(4.5),
            timestamp: 1234567891,
        },
        UserInteraction {
            user_id: user1,
            item_id: item3,
            interaction_type: InteractionType::View,
            rating: Some(4.0),
            timestamp: 1234567892,
        },
        // User 2 likes items 1, 2, 4 (similar to user 1)
        UserInteraction {
            user_id: user2,
            item_id: item1,
            interaction_type: InteractionType::Purchase,
            rating: Some(5.0),
            timestamp: 1234567893,
        },
        UserInteraction {
            user_id: user2,
            item_id: item2,
            interaction_type: InteractionType::Like,
            rating: Some(4.0),
            timestamp: 1234567894,
        },
        UserInteraction {
            user_id: user2,
            item_id: item4,
            interaction_type: InteractionType::Purchase,
            rating: Some(5.0),
            timestamp: 1234567895,
        },
        // User 3 likes items 4, 5 (different preferences)
        UserInteraction {
            user_id: user3,
            item_id: item4,
            interaction_type: InteractionType::Like,
            rating: Some(4.5),
            timestamp: 1234567896,
        },
        UserInteraction {
            user_id: user3,
            item_id: item5,
            interaction_type: InteractionType::Purchase,
            rating: Some(5.0),
            timestamp: 1234567897,
        },
    ];

    println!("Training recommendation model with {} interactions...", interactions.len());
    engine.train(interactions).await?;
    println!("✅ Model trained successfully!\n");

    // Get recommendations for user 1
    println!("Getting recommendations for User 1...");
    let request = RecommendationRequest {
        user_id: user1,
        limit: 5,
        item_type: None,
        exclude_items: Some(vec![item1, item2, item3]), // Exclude items user already has
    };

    match engine.recommend(request).await {
        Ok(response) => {
            println!("✅ Recommendations for User 1:");
            println!("   Algorithm: {}", response.algorithm);
            println!("   Confidence: {:.2}%", response.confidence * 100.0);
            println!("   Items:");
            for (i, item) in response.items.iter().enumerate() {
                println!("     {}. Item {} (score: {:.3}) - {}",
                    i + 1, item.item_id, item.score, item.reason);
            }
            println!();
        }
        Err(e) => {
            eprintln!("❌ Error getting recommendations: {}", e);
        }
    }

    // Get recommendations for user 3
    println!("Getting recommendations for User 3...");
    let request = RecommendationRequest {
        user_id: user3,
        limit: 5,
        item_type: None,
        exclude_items: Some(vec![item4, item5]),
    };

    match engine.recommend(request).await {
        Ok(response) => {
            println!("✅ Recommendations for User 3:");
            println!("   Algorithm: {}", response.algorithm);
            println!("   Confidence: {:.2}%", response.confidence * 100.0);
            println!("   Items:");
            for (i, item) in response.items.iter().enumerate() {
                println!("     {}. Item {} (score: {:.3}) - {}",
                    i + 1, item.item_id, item.score, item.reason);
            }
            println!();
        }
        Err(e) => {
            eprintln!("❌ Error getting recommendations: {}", e);
        }
    }

    println!("🎉 Demo completed successfully!");

    Ok(())
}
