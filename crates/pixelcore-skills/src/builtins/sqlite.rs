//! SQLite database operations skills

use async_trait::async_trait;
use rusqlite::{Connection, params_from_iter, types::ValueRef};
use serde_json::json;
use std::sync::Arc;

use crate::{Skill, SkillInput, SkillOutput, SkillError};

/// SQLite query skill - executes SELECT queries
pub struct SqliteQuerySkill;

#[async_trait]
impl Skill for SqliteQuerySkill {
    fn name(&self) -> &str {
        "sqlite_query"
    }

    fn description(&self) -> &str {
        "Execute a SELECT query on a SQLite database and return results as JSON"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "db_path": {
                    "type": "string",
                    "description": "Path to the SQLite database file"
                },
                "query": {
                    "type": "string",
                    "description": "SQL SELECT query to execute"
                },
                "params": {
                    "type": "array",
                    "description": "Query parameters (optional)",
                    "items": {
                        "type": ["string", "number", "boolean", "null"]
                    }
                }
            },
            "required": ["db_path", "query"]
        })
    }

    async fn execute(&self, input: SkillInput) -> Result<SkillOutput, SkillError> {
        let db_path = input.args.get("db_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::Execution("Missing 'db_path' parameter".to_string()))?;

        let query = input.args.get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::Execution("Missing 'query' parameter".to_string()))?;

        let params = input.args.get("params")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().map(|v| value_to_string(v)).collect::<Vec<_>>())
            .unwrap_or_default();

        match execute_query(db_path, query, &params) {
            Ok(rows) => Ok(SkillOutput {
                success: true,
                result: json!(rows),
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

/// SQLite execute skill - executes INSERT/UPDATE/DELETE statements
pub struct SqliteExecuteSkill;

#[async_trait]
impl Skill for SqliteExecuteSkill {
    fn name(&self) -> &str {
        "sqlite_execute"
    }

    fn description(&self) -> &str {
        "Execute an INSERT, UPDATE, or DELETE statement on a SQLite database"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "db_path": {
                    "type": "string",
                    "description": "Path to the SQLite database file"
                },
                "statement": {
                    "type": "string",
                    "description": "SQL statement to execute"
                },
                "params": {
                    "type": "array",
                    "description": "Statement parameters (optional)",
                    "items": {
                        "type": ["string", "number", "boolean", "null"]
                    }
                }
            },
            "required": ["db_path", "statement"]
        })
    }

    async fn execute(&self, input: SkillInput) -> Result<SkillOutput, SkillError> {
        let db_path = input.args.get("db_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::Execution("Missing 'db_path' parameter".to_string()))?;

        let statement = input.args.get("statement")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::Execution("Missing 'statement' parameter".to_string()))?;

        let params = input.args.get("params")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().map(|v| value_to_string(v)).collect::<Vec<_>>())
            .unwrap_or_default();

        match execute_statement(db_path, statement, &params) {
            Ok(rows_affected) => Ok(SkillOutput {
                success: true,
                result: json!({"rows_affected": rows_affected}),
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

/// Helper function to convert JSON value to string for SQLite parameters
fn value_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "NULL".to_string(),
        _ => value.to_string(),
    }
}

/// Execute a SELECT query and return results as JSON
fn execute_query(db_path: &str, query: &str, params: &[String]) -> Result<Vec<serde_json::Value>, String> {
    let conn = Connection::open(db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let mut stmt = conn.prepare(query)
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let column_count = stmt.column_count();
    let column_names: Vec<String> = (0..column_count)
        .map(|i| stmt.column_name(i).unwrap_or("").to_string())
        .collect();

    let rows = stmt.query_map(params_from_iter(params.iter()), |row| {
        let mut obj = serde_json::Map::new();
        for (i, name) in column_names.iter().enumerate() {
            let value_ref = row.get_ref(i).unwrap();
            let json_value = match value_ref {
                ValueRef::Null => json!(null),
                ValueRef::Integer(i) => json!(i),
                ValueRef::Real(f) => json!(f),
                ValueRef::Text(s) => json!(String::from_utf8_lossy(s).to_string()),
                ValueRef::Blob(b) => json!(b),
            };
            obj.insert(name.clone(), json_value);
        }
        Ok(json!(obj))
    }).map_err(|e| format!("Failed to execute query: {}", e))?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| format!("Failed to read row: {}", e))?);
    }

    Ok(result)
}

/// Execute an INSERT/UPDATE/DELETE statement
fn execute_statement(db_path: &str, statement: &str, params: &[String]) -> Result<usize, String> {
    let conn = Connection::open(db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let rows_affected = conn.execute(statement, params_from_iter(params.iter()))
        .map_err(|e| format!("Failed to execute statement: {}", e))?;

    Ok(rows_affected)
}

/// Create SQLite skills
pub fn create_sqlite_skills() -> Vec<Arc<dyn Skill>> {
    vec![
        Arc::new(SqliteQuerySkill),
        Arc::new(SqliteExecuteSkill),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn setup_test_db() -> NamedTempFile {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_str().unwrap();

        let conn = Connection::open(db_path).unwrap();
        conn.execute(
            "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)",
            [],
        ).unwrap();
        conn.execute("INSERT INTO users (name, age) VALUES ('Alice', 30)", []).unwrap();
        conn.execute("INSERT INTO users (name, age) VALUES ('Bob', 25)", []).unwrap();

        temp_file
    }

    #[tokio::test]
    async fn test_sqlite_query() {
        let temp_file = setup_test_db();
        let db_path = temp_file.path().to_str().unwrap();

        let skill = SqliteQuerySkill;
        let input = SkillInput {
            name: "sqlite_query".to_string(),
            args: json!({
                "db_path": db_path,
                "query": "SELECT * FROM users WHERE age > 26"
            }),
        };

        let output = skill.execute(input).await.unwrap();
        assert!(output.success);
        let rows = output.result.as_array().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["name"], "Alice");
        assert_eq!(rows[0]["age"], 30);
    }

    #[tokio::test]
    async fn test_sqlite_execute() {
        let temp_file = setup_test_db();
        let db_path = temp_file.path().to_str().unwrap();

        let skill = SqliteExecuteSkill;
        let input = SkillInput {
            name: "sqlite_execute".to_string(),
            args: json!({
                "db_path": db_path,
                "statement": "INSERT INTO users (name, age) VALUES ('Charlie', 35)"
            }),
        };

        let output = skill.execute(input).await.unwrap();
        assert!(output.success);
        assert_eq!(output.result["rows_affected"], 1);
    }
}
