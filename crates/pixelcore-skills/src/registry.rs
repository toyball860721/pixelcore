use std::collections::HashMap;
use std::sync::Arc;
use crate::skill::Skill;
use crate::error::SkillError;
use pixelcore_claw::types::Tool;

pub struct SkillRegistry {
    skills: HashMap<String, Arc<dyn Skill>>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self { skills: HashMap::new() }
    }

    pub fn register(&mut self, skill: Arc<dyn Skill>) {
        self.skills.insert(skill.name().to_string(), skill);
    }

    pub fn get(&self, name: &str) -> Result<Arc<dyn Skill>, SkillError> {
        self.skills
            .get(name)
            .cloned()
            .ok_or_else(|| SkillError::NotFound(name.to_string()))
    }

    pub fn list(&self) -> Vec<&str> {
        self.skills.keys().map(|s| s.as_str()).collect()
    }

    /// Convert all registered skills to `Tool` descriptors for LLM requests.
    pub fn as_tools(&self) -> Vec<Tool> {
        self.skills.values().map(|s| Tool {
            name: s.name().to_string(),
            description: s.description().to_string(),
            input_schema: s.input_schema(),
        }).collect()
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}
