use crate::models::{AgentListing, AgentStatus};
use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Agent 注册表存储
pub struct RegistryStorage {
    conn: Arc<Mutex<Connection>>,
}

impl RegistryStorage {
    /// 创建新的存储实例
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path)
            .context("Failed to open database")?;

        let storage = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        storage.init_schema()?;
        Ok(storage)
    }

    /// 创建内存数据库 (用于测试)
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .context("Failed to create in-memory database")?;

        let storage = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        storage.init_schema()?;
        Ok(storage)
    }

    /// 初始化数据库 schema
    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS agent_listings (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                version TEXT NOT NULL,
                owner_id TEXT NOT NULL,
                capabilities TEXT NOT NULL,
                pricing TEXT NOT NULL,
                sla TEXT NOT NULL,
                status TEXT NOT NULL,
                reputation_score REAL NOT NULL DEFAULT 0.0,
                total_transactions INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        ).context("Failed to create agent_listings table")?;

        // 创建索引
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_owner_id ON agent_listings(owner_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_status ON agent_listings(status)",
            [],
        )?;

        Ok(())
    }

    /// 保存 Agent 列表
    pub fn save(&self, listing: &AgentListing) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        let capabilities_json = serde_json::to_string(&listing.capabilities)?;
        let pricing_json = serde_json::to_string(&listing.pricing)?;
        let sla_json = serde_json::to_string(&listing.sla)?;
        let status_str = format!("{:?}", listing.status);

        conn.execute(
            "INSERT OR REPLACE INTO agent_listings
            (id, name, description, version, owner_id, capabilities, pricing, sla,
             status, reputation_score, total_transactions, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                listing.id.to_string(),
                listing.name,
                listing.description,
                listing.version,
                listing.owner_id.to_string(),
                capabilities_json,
                pricing_json,
                sla_json,
                status_str,
                listing.reputation_score,
                listing.total_transactions as i64,
                listing.created_at.to_rfc3339(),
                listing.updated_at.to_rfc3339(),
            ],
        ).context("Failed to save agent listing")?;

        Ok(())
    }

    /// 根据 ID 获取 Agent
    pub fn get(&self, id: &Uuid) -> Result<Option<AgentListing>> {
        let conn = self.conn.lock().unwrap();

        let result = conn.query_row(
            "SELECT id, name, description, version, owner_id, capabilities, pricing, sla,
                    status, reputation_score, total_transactions, created_at, updated_at
             FROM agent_listings WHERE id = ?1",
            params![id.to_string()],
            |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let description: String = row.get(2)?;
                let version: String = row.get(3)?;
                let owner_id: String = row.get(4)?;
                let capabilities_json: String = row.get(5)?;
                let pricing_json: String = row.get(6)?;
                let sla_json: String = row.get(7)?;
                let status_str: String = row.get(8)?;
                let reputation_score: f64 = row.get(9)?;
                let total_transactions: i64 = row.get(10)?;
                let created_at: String = row.get(11)?;
                let updated_at: String = row.get(12)?;

                Ok((
                    id, name, description, version, owner_id,
                    capabilities_json, pricing_json, sla_json, status_str,
                    reputation_score, total_transactions, created_at, updated_at
                ))
            },
        ).optional()?;

        if let Some((
            id, name, description, version, owner_id,
            capabilities_json, pricing_json, sla_json, status_str,
            reputation_score, total_transactions, created_at, updated_at
        )) = result {
            let listing = AgentListing {
                id: Uuid::parse_str(&id)?,
                name,
                description,
                version,
                owner_id: Uuid::parse_str(&owner_id)?,
                capabilities: serde_json::from_str(&capabilities_json)?,
                pricing: serde_json::from_str(&pricing_json)?,
                sla: serde_json::from_str(&sla_json)?,
                status: match status_str.as_str() {
                    "Draft" => AgentStatus::Draft,
                    "Published" => AgentStatus::Published,
                    "Paused" => AgentStatus::Paused,
                    "Archived" => AgentStatus::Archived,
                    _ => AgentStatus::Draft,
                },
                reputation_score,
                total_transactions: total_transactions as u64,
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at)?.with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at)?.with_timezone(&chrono::Utc),
            };

            Ok(Some(listing))
        } else {
            Ok(None)
        }
    }

    /// 删除 Agent
    pub fn delete(&self, id: &Uuid) -> Result<bool> {
        let conn = self.conn.lock().unwrap();

        let rows_affected = conn.execute(
            "DELETE FROM agent_listings WHERE id = ?1",
            params![id.to_string()],
        )?;

        Ok(rows_affected > 0)
    }

    /// 列出所有 Agent (带分页)
    pub fn list(&self, offset: usize, limit: usize) -> Result<Vec<AgentListing>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, name, description, version, owner_id, capabilities, pricing, sla,
                    status, reputation_score, total_transactions, created_at, updated_at
             FROM agent_listings
             ORDER BY created_at DESC
             LIMIT ?1 OFFSET ?2"
        )?;

        let listings = stmt.query_map(params![limit as i64, offset as i64], |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let description: String = row.get(2)?;
            let version: String = row.get(3)?;
            let owner_id: String = row.get(4)?;
            let capabilities_json: String = row.get(5)?;
            let pricing_json: String = row.get(6)?;
            let sla_json: String = row.get(7)?;
            let status_str: String = row.get(8)?;
            let reputation_score: f64 = row.get(9)?;
            let total_transactions: i64 = row.get(10)?;
            let created_at: String = row.get(11)?;
            let updated_at: String = row.get(12)?;

            Ok((
                id, name, description, version, owner_id,
                capabilities_json, pricing_json, sla_json, status_str,
                reputation_score, total_transactions, created_at, updated_at
            ))
        })?
        .filter_map(|result| {
            result.ok().and_then(|(
                id, name, description, version, owner_id,
                capabilities_json, pricing_json, sla_json, status_str,
                reputation_score, total_transactions, created_at, updated_at
            )| {
                Some(AgentListing {
                    id: Uuid::parse_str(&id).ok()?,
                    name,
                    description,
                    version,
                    owner_id: Uuid::parse_str(&owner_id).ok()?,
                    capabilities: serde_json::from_str(&capabilities_json).ok()?,
                    pricing: serde_json::from_str(&pricing_json).ok()?,
                    sla: serde_json::from_str(&sla_json).ok()?,
                    status: match status_str.as_str() {
                        "Draft" => AgentStatus::Draft,
                        "Published" => AgentStatus::Published,
                        "Paused" => AgentStatus::Paused,
                        "Archived" => AgentStatus::Archived,
                        _ => AgentStatus::Draft,
                    },
                    reputation_score,
                    total_transactions: total_transactions as u64,
                    created_at: chrono::DateTime::parse_from_rfc3339(&created_at).ok()?.with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at).ok()?.with_timezone(&chrono::Utc),
                })
            })
        })
        .collect();

        Ok(listings)
    }

    /// 统计 Agent 数量
    pub fn count(&self) -> Result<usize> {
        let conn = self.conn.lock().unwrap();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM agent_listings",
            [],
            |row| row.get(0),
        )?;

        Ok(count as usize)
    }
}
