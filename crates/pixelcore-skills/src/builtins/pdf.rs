//! PDF document processing skills

use async_trait::async_trait;
use pdf_extract::extract_text;
use serde_json::json;
use std::sync::Arc;

use crate::{Skill, SkillInput, SkillOutput, SkillError};

/// PDF extract skill - extracts text content from PDF files
pub struct PdfExtractSkill;

#[async_trait]
impl Skill for PdfExtractSkill {
    fn name(&self) -> &str {
        "pdf_extract"
    }

    fn description(&self) -> &str {
        "Extract text content from a PDF file"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the PDF file"
                }
            },
            "required": ["file_path"]
        })
    }

    async fn execute(&self, input: SkillInput) -> Result<SkillOutput, SkillError> {
        let file_path = input.args.get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::Execution("Missing 'file_path' parameter".to_string()))?;

        match extract_pdf_text(file_path) {
            Ok(text) => Ok(SkillOutput {
                success: true,
                result: json!({
                    "text": text,
                    "length": text.len()
                }),
                error: None,
            }),
            Err(e) => Ok(SkillOutput {
                success: false,
                result: json!(null),
                error: Some(e),
            }),
        }
    }
}

/// Extract text from PDF file
fn extract_pdf_text(file_path: &str) -> Result<String, String> {
    extract_text(file_path)
        .map_err(|e| format!("Failed to extract text from PDF: {}", e))
}

/// Create PDF skills
pub fn create_pdf_skills() -> Vec<Arc<dyn Skill>> {
    vec![
        Arc::new(PdfExtractSkill),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pdf_extract_basic() {
        // Note: This test requires a valid PDF file
        // For now, we'll test the skill structure
        let skill = PdfExtractSkill;

        // Test with a non-existent file to verify error handling
        let input = SkillInput {
            name: "pdf_extract".to_string(),
            args: json!({
                "file_path": "/nonexistent/file.pdf"
            }),
        };

        let output = skill.execute(input).await.unwrap();
        assert!(!output.success);
        assert!(output.error.is_some());
    }
}
