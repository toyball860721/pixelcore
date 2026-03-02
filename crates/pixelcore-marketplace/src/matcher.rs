use pixelcore_registry::AgentListing;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 匹配结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResult {
    /// Agent ID
    pub agent_id: Uuid,
    /// 匹配分数 (0.0 - 1.0)
    pub score: f64,
    /// 匹配原因
    pub reasons: Vec<String>,
}

/// 智能匹配器
pub struct SmartMatcher;

impl SmartMatcher {
    /// 创建新的智能匹配器
    pub fn new() -> Self {
        Self
    }

    /// 计算 Agent 与需求的匹配度
    pub fn calculate_match_score(
        &self,
        agent: &AgentListing,
        required_skills: &[String],
        optional_skills: &[String],
    ) -> MatchResult {
        let mut score = 0.0;
        let mut reasons = Vec::new();

        // 1. 必需技能匹配 (权重 50%)
        let required_match_count = required_skills.iter()
            .filter(|skill| self.has_skill(agent, skill))
            .count();

        if !required_skills.is_empty() {
            let required_score = required_match_count as f64 / required_skills.len() as f64;
            score += required_score * 0.5;

            if required_match_count == required_skills.len() {
                reasons.push("满足所有必需技能".to_string());
            } else {
                reasons.push(format!(
                    "满足 {}/{} 必需技能",
                    required_match_count,
                    required_skills.len()
                ));
            }
        }

        // 2. 可选技能匹配 (权重 20%)
        if !optional_skills.is_empty() {
            let optional_match_count = optional_skills.iter()
                .filter(|skill| self.has_skill(agent, skill))
                .count();

            let optional_score = optional_match_count as f64 / optional_skills.len() as f64;
            score += optional_score * 0.2;

            if optional_match_count > 0 {
                reasons.push(format!(
                    "额外支持 {} 个可选技能",
                    optional_match_count
                ));
            }
        }

        // 3. 信誉分数 (权重 20%)
        let reputation_score = agent.reputation_score / 5.0; // 归一化到 0-1
        score += reputation_score * 0.2;

        if agent.reputation_score >= 4.5 {
            reasons.push("高信誉 (4.5+)".to_string());
        } else if agent.reputation_score >= 4.0 {
            reasons.push("良好信誉 (4.0+)".to_string());
        }

        // 4. 交易历史 (权重 10%)
        let transaction_score = if agent.total_transactions > 100 {
            1.0
        } else if agent.total_transactions > 10 {
            0.7
        } else if agent.total_transactions > 0 {
            0.3
        } else {
            0.0
        };
        score += transaction_score * 0.1;

        if agent.total_transactions > 100 {
            reasons.push("丰富的交易经验 (100+)".to_string());
        } else if agent.total_transactions > 10 {
            reasons.push("有一定交易经验".to_string());
        }

        MatchResult {
            agent_id: agent.id,
            score,
            reasons,
        }
    }

    /// 对多个 Agent 进行匹配和排序
    pub fn match_and_rank(
        &self,
        agents: &[AgentListing],
        required_skills: &[String],
        optional_skills: &[String],
    ) -> Vec<(AgentListing, MatchResult)> {
        let mut results: Vec<(AgentListing, MatchResult)> = agents
            .iter()
            .map(|agent| {
                let match_result = self.calculate_match_score(
                    agent,
                    required_skills,
                    optional_skills,
                );
                (agent.clone(), match_result)
            })
            .collect();

        // 按匹配分数降序排序
        results.sort_by(|a, b| {
            b.1.score.partial_cmp(&a.1.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    /// 推荐最佳 Agent
    pub fn recommend_best(
        &self,
        agents: &[AgentListing],
        required_skills: &[String],
        optional_skills: &[String],
        top_n: usize,
    ) -> Vec<(AgentListing, MatchResult)> {
        let mut ranked = self.match_and_rank(agents, required_skills, optional_skills);

        // 只保留匹配分数 > 0.5 的结果
        ranked.retain(|(_, result)| result.score > 0.5);

        // 取前 N 个
        ranked.truncate(top_n);

        ranked
    }

    /// 检查 Agent 是否有指定技能
    fn has_skill(&self, agent: &AgentListing, skill_name: &str) -> bool {
        agent.capabilities.iter().any(|cap| {
            cap.skill_name.to_lowercase() == skill_name.to_lowercase()
        })
    }

    /// 计算相似度 (基于技能集合)
    pub fn calculate_similarity(&self, agent1: &AgentListing, agent2: &AgentListing) -> f64 {
        let skills1: std::collections::HashSet<String> = agent1.capabilities
            .iter()
            .map(|c| c.skill_name.to_lowercase())
            .collect();

        let skills2: std::collections::HashSet<String> = agent2.capabilities
            .iter()
            .map(|c| c.skill_name.to_lowercase())
            .collect();

        // Jaccard 相似度
        let intersection = skills1.intersection(&skills2).count();
        let union = skills1.union(&skills2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// 推荐相似的 Agent
    pub fn recommend_similar(
        &self,
        target_agent: &AgentListing,
        all_agents: &[AgentListing],
        top_n: usize,
    ) -> Vec<(AgentListing, f64)> {
        let mut similarities: Vec<(AgentListing, f64)> = all_agents
            .iter()
            .filter(|agent| agent.id != target_agent.id) // 排除自己
            .map(|agent| {
                let similarity = self.calculate_similarity(target_agent, agent);
                (agent.clone(), similarity)
            })
            .collect();

        // 按相似度降序排序
        similarities.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 取前 N 个
        similarities.truncate(top_n);

        similarities
    }
}

impl Default for SmartMatcher {
    fn default() -> Self {
        Self::new()
    }
}
