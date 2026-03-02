use crate::models::{ReputationRecord, Review};
use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// 信誉存储
pub struct ReputationStorage {
    conn: Arc<Mutex<Connection>>,
}

impl ReputationStorage {
    /// 创建新的存储实例
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let storage = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        storage.init_schema()?;
        Ok(storage)
    }

    /// 创建内存数据库
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
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
            "CREATE TABLE IF NOT EXISTS reputation_records (
                agent_id TEXT PRIMARY KEY,
                score REAL NOT NULL,
                total_transactions INTEGER NOT NULL,
                successful_transactions INTEGER NOT NULL,
                average_response_time_ms INTEGER NOT NULL,
                level TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS reviews (
                id TEXT PRIMARY KEY,
                transaction_id TEXT NOT NULL,
                agent_id TEXT NOT NULL,
                reviewer_id TEXT NOT NULL,
                rating INTEGER NOT NULL,
                comment TEXT NOT NULL,
                verified INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (agent_id) REFERENCES reputation_records(agent_id)
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_reviews_agent ON reviews(agent_id)",
            [],
        )?;

        Ok(())
    }

    /// 保存信誉记录
    pub fn save_record(&self, record: &ReputationRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT OR REPLACE INTO reputation_records
            (agent_id, score, total_transactions, successful_transactions,
             average_response_time_ms, level, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                record.agent_id.to_string(),
                record.score,
                record.total_transactions as i64,
                record.successful_transactions as i64,
                record.average_response_time_ms as i64,
                format!("{:?}", record.level),
                record.updated_at.to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    /// 获取信誉记录
    pub fn get_record(&self, agent_id: &Uuid) -> Result<Option<ReputationRecord>> {
        let conn = self.conn.lock().unwrap();

        let record = conn.query_row(
            "SELECT agent_id, score, total_transactions, successful_transactions,
                    average_response_time_ms, level, updated_at
             FROM reputation_records WHERE agent_id = ?1",
            params![agent_id.to_string()],
            |row| {
                let agent_id: String = row.get(0)?;
                let score: f64 = row.get(1)?;
                let total_transactions: i64 = row.get(2)?;
                let successful_transactions: i64 = row.get(3)?;
                let average_response_time_ms: i64 = row.get(4)?;
                let level_str: String = row.get(5)?;
                let updated_at: String = row.get(6)?;

                Ok((agent_id, score, total_transactions, successful_transactions,
                    average_response_time_ms, level_str, updated_at))
            },
        ).optional()?;

        if let Some((agent_id, score, total_transactions, successful_transactions,
                     average_response_time_ms, level_str, updated_at)) = record {

            let level = match level_str.as_str() {
                "Newcomer" => crate::models::ReputationLevel::Newcomer,
                "Regular" => crate::models::ReputationLevel::Regular,
                "Excellent" => crate::models::ReputationLevel::Excellent,
                "Expert" => crate::models::ReputationLevel::Expert,
                _ => crate::models::ReputationLevel::Newcomer,
            };

            let reviews = self.get_reviews_for_agent(&Uuid::parse_str(&agent_id)?)?;

            Ok(Some(ReputationRecord {
                agent_id: Uuid::parse_str(&agent_id)?,
                score,
                total_transactions: total_transactions as u64,
                successful_transactions: successful_transactions as u64,
                average_response_time_ms: average_response_time_ms as u64,
                level,
                reviews,
                updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at)?
                    .with_timezone(&chrono::Utc),
            }))
        } else {
            Ok(None)
        }
    }

    /// 保存评价
    pub fn save_review(&self, review: &Review) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT OR REPLACE INTO reviews
            (id, transaction_id, agent_id, reviewer_id, rating, comment, verified, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                review.id.to_string(),
                review.transaction_id.to_string(),
                review.agent_id.to_string(),
                review.reviewer_id.to_string(),
                review.rating as i32,
                review.comment,
                if review.verified { 1 } else { 0 },
                review.created_at.to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    /// 获取 Agent 的所有评价
    fn get_reviews_for_agent(&self, agent_id: &Uuid) -> Result<Vec<Review>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, transaction_id, agent_id, reviewer_id, rating, comment, verified, created_at
             FROM reviews WHERE agent_id = ?1 ORDER BY created_at DESC"
        )?;

        let reviews = stmt.query_map(params![agent_id.to_string()], |row| {
            let id: String = row.get(0)?;
            let transaction_id: String = row.get(1)?;
            let agent_id: String = row.get(2)?;
            let reviewer_id: String = row.get(3)?;
            let rating: i32 = row.get(4)?;
            let comment: String = row.get(5)?;
            let verified: i32 = row.get(6)?;
            let created_at: String = row.get(7)?;

            Ok((id, transaction_id, agent_id, reviewer_id, rating, comment, verified, created_at))
        })?
        .filter_map(|result| {
            result.ok().and_then(|(id, transaction_id, agent_id, reviewer_id, rating, comment, verified, created_at)| {
                Some(Review {
                    id: Uuid::parse_str(&id).ok()?,
                    transaction_id: Uuid::parse_str(&transaction_id).ok()?,
                    agent_id: Uuid::parse_str(&agent_id).ok()?,
                    reviewer_id: Uuid::parse_str(&reviewer_id).ok()?,
                    rating: rating as u8,
                    comment,
                    verified: verified != 0,
                    created_at: chrono::DateTime::parse_from_rfc3339(&created_at).ok()?
                        .with_timezone(&chrono::Utc),
                })
            })
        })
        .collect();

        Ok(reviews)
    }
}
