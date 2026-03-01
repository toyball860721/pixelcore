//! PDF text extraction demo
//!
//! This example demonstrates how to use PDF skills to:
//! - Extract text content from PDF files
//! - Get text length information
//!
//! Note: This example requires a PDF file to work with.
//! You can test it with any PDF file you have.

use pixelcore_skills::{create_pdf_skills, SkillInput};
use serde_json::json;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PDF Text Extraction Demo ===\n");

    // Get PDF skills
    let skills = create_pdf_skills();
    let pdf_extract_skill = &skills[0];

    // Check if a PDF file path was provided as argument
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: cargo run --example pdf_demo <path_to_pdf_file>");
        println!("\nExample:");
        println!("  cargo run --example pdf_demo /path/to/document.pdf");
        println!("\nThis demo will extract all text content from the PDF file.");
        return Ok(());
    }

    let pdf_path = &args[1];
    println!("Extracting text from: {}\n", pdf_path);

    // Extract text from PDF
    let extract_input = SkillInput {
        name: "pdf_extract".to_string(),
        args: json!({
            "file_path": pdf_path
        }),
    };

    let result = pdf_extract_skill.execute(extract_input).await?;

    if result.success {
        let text = result.result["text"].as_str().unwrap_or("");
        let length = result.result["length"].as_u64().unwrap_or(0);

        println!("✓ Successfully extracted text from PDF");
        println!("  Total characters: {}", length);
        println!("\n--- Extracted Text ---\n");

        // Print first 1000 characters if text is long
        if text.len() > 1000 {
            println!("{}", &text[..1000]);
            println!("\n... (showing first 1000 characters of {} total)", length);
        } else {
            println!("{}", text);
        }

        println!("\n--- End of Text ---\n");

        // Show some statistics
        let lines = text.lines().count();
        let words = text.split_whitespace().count();

        println!("Statistics:");
        println!("  Lines: {}", lines);
        println!("  Words: {}", words);
        println!("  Characters: {}", length);
    } else {
        println!("✗ Failed to extract text from PDF");
        if let Some(error) = result.error {
            println!("  Error: {}", error);
        }
    }

    println!("\n=== Demo Complete ===");

    Ok(())
}
