use thiserror::Error;

#[derive(Debug, Error)]
pub enum SkillError {
    #[error("Skill not found: {0}")]
    NotFound(String),

    #[error("Execution error: {0}")]
    Execution(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}
