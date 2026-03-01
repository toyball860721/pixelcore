//! Skill preloader demo
//!
//! This example demonstrates how to preload commonly used skills
//! at system startup to reduce first-use latency.

use pixelcore_runtime::{SkillPreloader, PreloaderConfig};
use std::time::Duration;

/// Simulate skill loading
async fn load_skill(skill_name: String) -> Result<(), String> {
    println!("  Loading skill: {}", skill_name);

    // Simulate different loading times for different skills
    let delay = match skill_name.as_str() {
        "echo" => 10,
        "calculate" => 20,
        "json_parse" => 30,
        "http_fetch" => 50,
        "database_query" => 100,
        "failing_skill" => {
            // Simulate a skill that fails to load
            tokio::time::sleep(Duration::from_millis(10)).await;
            return Err("Failed to initialize database connection".to_string());
        }
        _ => 15,
    };

    tokio::time::sleep(Duration::from_millis(delay)).await;
    println!("    ✓ Loaded {} ({}ms)", skill_name, delay);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Skill Preloader Demo ===\n");

    // Demo 1: Basic preloading
    println!("Demo 1: Basic Skill Preloading");
    println!("─────────────────────────────");

    let config = PreloaderConfig {
        skill_names: vec![
            "echo".to_string(),
            "calculate".to_string(),
            "json_parse".to_string(),
            "http_fetch".to_string(),
        ],
        enable_warmup: true,
        warmup_timeout_ms: 5000,
    };

    let preloader = SkillPreloader::new(config);

    println!("Preloading {} skills...\n", 4);
    let start = std::time::Instant::now();
    let stats = preloader.preload(load_skill).await;
    let duration = start.elapsed();

    println!("\nPreload completed in {:.2}ms", duration.as_millis());
    println!("Results:");
    println!("  Total skills: {}", stats.total_skills);
    println!("  Loaded: {}", stats.loaded_skills);
    println!("  Failed: {}", stats.failed_skills);
    println!("  Success rate: {:.1}%\n", stats.success_rate() * 100.0);

    // Demo 2: Preloading with failures
    println!("Demo 2: Preloading with Failures");
    println!("─────────────────────────────");

    let config2 = PreloaderConfig {
        skill_names: vec![
            "echo".to_string(),
            "calculate".to_string(),
            "failing_skill".to_string(),
            "json_parse".to_string(),
        ],
        enable_warmup: true,
        warmup_timeout_ms: 5000,
    };

    let preloader2 = SkillPreloader::new(config2);

    println!("Preloading {} skills (including one that fails)...\n", 4);
    let stats2 = preloader2.preload(load_skill).await;

    println!("\nPreload completed");
    println!("Results:");
    println!("  Total skills: {}", stats2.total_skills);
    println!("  Loaded: {}", stats2.loaded_skills);
    println!("  Failed: {}", stats2.failed_skills);
    println!("  Success rate: {:.1}%", stats2.success_rate() * 100.0);

    println!("\nDetailed results:");
    for result in &stats2.results {
        if result.success {
            println!("  ✓ {} - {}ms", result.skill_name, result.duration_ms);
        } else {
            println!(
                "  ✗ {} - {}ms (error: {})",
                result.skill_name,
                result.duration_ms,
                result.error.as_ref().unwrap_or(&"Unknown error".to_string())
            );
        }
    }

    // Demo 3: Check preloaded skills
    println!("\nDemo 3: Checking Preloaded Skills");
    println!("─────────────────────────────");

    let preloaded_names = preloader2.preloaded_skill_names().await;
    println!("Successfully preloaded skills:");
    for name in &preloaded_names {
        println!("  • {}", name);
    }

    // Check specific skills
    println!("\nChecking specific skills:");
    let skills_to_check = vec!["echo", "failing_skill", "json_parse"];
    for skill in skills_to_check {
        let is_preloaded = preloader2.is_preloaded(skill).await;
        println!(
            "  {} is {}",
            skill,
            if is_preloaded { "preloaded ✓" } else { "not preloaded ✗" }
        );
    }

    // Demo 4: Performance comparison
    println!("\nDemo 4: Performance Comparison");
    println!("─────────────────────────────");

    let config3 = PreloaderConfig {
        skill_names: vec![
            "echo".to_string(),
            "calculate".to_string(),
            "json_parse".to_string(),
            "http_fetch".to_string(),
            "database_query".to_string(),
        ],
        enable_warmup: true,
        warmup_timeout_ms: 5000,
    };

    let preloader3 = SkillPreloader::new(config3);

    println!("Preloading 5 skills...");
    let start = std::time::Instant::now();
    let stats3 = preloader3.preload(load_skill).await;
    let duration = start.elapsed();

    println!("\nPerformance metrics:");
    println!("  Total time: {:.2}ms", duration.as_millis());
    println!("  Average per skill: {:.2}ms", stats3.total_duration_ms as f64 / stats3.total_skills as f64);
    println!("  Fastest skill: {}ms", stats3.results.iter().map(|r| r.duration_ms).min().unwrap_or(0));
    println!("  Slowest skill: {}ms", stats3.results.iter().map(|r| r.duration_ms).max().unwrap_or(0));

    println!("\n=== Demo completed successfully ===");
    Ok(())
}
