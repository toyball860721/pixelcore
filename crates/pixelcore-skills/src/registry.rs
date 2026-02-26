use std::collections::HashMap;
use std::sync::Arc;
use crate::skill::Skill;
use crate::error::SkillError;

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
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}
