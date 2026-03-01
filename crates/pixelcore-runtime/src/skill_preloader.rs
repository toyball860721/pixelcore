//! Skill preloader for reducing first-use latency
//!
//! This module provides functionality to preload commonly used skills
//! at system startup, reducing the latency of first skill execution.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Preloader configuration
#[derive(Debug, Clone)]
pub struct PreloaderConfig {
    /// List of skill names to preload
    pub skill_names: Vec<String>,
    /// Whether to warmup skills (execute once to load dependencies)
    pub enable_warmup: bool,
    /// Timeout for each skill warmup (milliseconds)
    pub warmup_timeout_ms: u64,
}

impl Default for PreloaderConfig {
    fn default() -> Self {
        Self {
            skill_names: vec![
                "echo".to_string(),
                "calculate".to_string(),
                "json_parse".to_string(),
                "http_fetch".to_string(),
            ],
            enable_warmup: true,
            warmup_timeout_ms: 5000,
        }
    }
}

/// Preload result for a single skill
#[derive(Debug, Clone)]
pub struct PreloadResult {
    pub skill_name: String,
    pub success: bool,
    pub duration_ms: u64,
    pub error: Option<String>,
}

/// Preloader statistics
#[derive(Debug, Clone, Default)]
pub struct PreloaderStats {
    pub total_skills: usize,
    pub loaded_skills: usize,
    pub failed_skills: usize,
    pub total_duration_ms: u64,
    pub results: Vec<PreloadResult>,
}

impl PreloaderStats {
    pub fn success_rate(&self) -> f64 {
        if self.total_skills == 0 {
            0.0
        } else {
            self.loaded_skills as f64 / self.total_skills as f64
        }
    }
}

/// Skill preloader
pub struct SkillPreloader {
    config: PreloaderConfig,
    stats: Arc<RwLock<PreloaderStats>>,
    preloaded_skills: Arc<RwLock<HashMap<String, bool>>>,
}

impl SkillPreloader {
    /// Create a new skill preloader
    pub fn new(config: PreloaderConfig) -> Self {
        Self {
            config,
            stats: Arc::new(RwLock::new(PreloaderStats::default())),
            preloaded_skills: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Preload skills
    pub async fn preload<F, Fut>(&self, loader: F) -> PreloaderStats
    where
        F: Fn(String) -> Fut,
        Fut: std::future::Future<Output = Result<(), String>>,
    {
        let start = Instant::now();
        let mut results = Vec::new();

        for skill_name in &self.config.skill_names {
            let skill_start = Instant::now();
            let result = loader(skill_name.clone()).await;
            let duration = skill_start.elapsed();

            let preload_result = PreloadResult {
                skill_name: skill_name.clone(),
                success: result.is_ok(),
                duration_ms: duration.as_millis() as u64,
                error: result.err(),
            };

            results.push(preload_result.clone());

            // Update preloaded skills map
            let mut preloaded = self.preloaded_skills.write().await;
            preloaded.insert(skill_name.clone(), preload_result.success);
        }

        let total_duration = start.elapsed();

        let loaded_skills = results.iter().filter(|r| r.success).count();
        let failed_skills = results.iter().filter(|r| !r.success).count();

        let stats = PreloaderStats {
            total_skills: self.config.skill_names.len(),
            loaded_skills,
            failed_skills,
            total_duration_ms: total_duration.as_millis() as u64,
            results,
        };

        // Update stats
        let mut stats_guard = self.stats.write().await;
        *stats_guard = stats.clone();

        stats
    }

    /// Check if a skill is preloaded
    pub async fn is_preloaded(&self, skill_name: &str) -> bool {
        let preloaded = self.preloaded_skills.read().await;
        preloaded.get(skill_name).copied().unwrap_or(false)
    }

    /// Get preloader statistics
    pub async fn stats(&self) -> PreloaderStats {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Get list of preloaded skill names
    pub async fn preloaded_skill_names(&self) -> Vec<String> {
        let preloaded = self.preloaded_skills.read().await;
        preloaded
            .iter()
            .filter(|(_, &success)| success)
            .map(|(name, _)| name.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_preloader_basic() {
        let config = PreloaderConfig {
            skill_names: vec!["skill1".to_string(), "skill2".to_string()],
            enable_warmup: false,
            warmup_timeout_ms: 1000,
        };

        let preloader = SkillPreloader::new(config);

        // Mock loader that always succeeds
        let loader = |_skill_name: String| async move { Ok::<(), String>(()) };

        let stats = preloader.preload(loader).await;

        assert_eq!(stats.total_skills, 2);
        assert_eq!(stats.loaded_skills, 2);
        assert_eq!(stats.failed_skills, 0);
        assert_eq!(stats.success_rate(), 1.0);
    }

    #[tokio::test]
    async fn test_preloader_with_failures() {
        let config = PreloaderConfig {
            skill_names: vec![
                "skill1".to_string(),
                "skill2".to_string(),
                "skill3".to_string(),
            ],
            enable_warmup: false,
            warmup_timeout_ms: 1000,
        };

        let preloader = SkillPreloader::new(config);

        // Mock loader that fails for skill2
        let loader = |skill_name: String| async move {
            if skill_name == "skill2" {
                Err("Failed to load".to_string())
            } else {
                Ok(())
            }
        };

        let stats = preloader.preload(loader).await;

        assert_eq!(stats.total_skills, 3);
        assert_eq!(stats.loaded_skills, 2);
        assert_eq!(stats.failed_skills, 1);
        assert!((stats.success_rate() - 0.666).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_is_preloaded() {
        let config = PreloaderConfig {
            skill_names: vec!["skill1".to_string()],
            enable_warmup: false,
            warmup_timeout_ms: 1000,
        };

        let preloader = SkillPreloader::new(config);

        // Initially not preloaded
        assert!(!preloader.is_preloaded("skill1").await);

        // Preload
        let loader = |_skill_name: String| async move { Ok::<(), String>(()) };
        preloader.preload(loader).await;

        // Now preloaded
        assert!(preloader.is_preloaded("skill1").await);
    }

    #[tokio::test]
    async fn test_preloaded_skill_names() {
        let config = PreloaderConfig {
            skill_names: vec![
                "skill1".to_string(),
                "skill2".to_string(),
                "skill3".to_string(),
            ],
            enable_warmup: false,
            warmup_timeout_ms: 1000,
        };

        let preloader = SkillPreloader::new(config);

        // Mock loader that fails for skill2
        let loader = |skill_name: String| async move {
            if skill_name == "skill2" {
                Err("Failed".to_string())
            } else {
                Ok(())
            }
        };

        preloader.preload(loader).await;

        let preloaded_names = preloader.preloaded_skill_names().await;
        assert_eq!(preloaded_names.len(), 2);
        assert!(preloaded_names.contains(&"skill1".to_string()));
        assert!(preloaded_names.contains(&"skill3".to_string()));
        assert!(!preloaded_names.contains(&"skill2".to_string()));
    }
}
