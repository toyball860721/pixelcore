//! PostgreSQL database operations skills

use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;
use tokio_postgres::{NoTls, Row};

use crate::{Skill, SkillInput, SkillOutput, SkillError};

/// PostgreSQL query skill - executes SELECT queries
pub struct PostgresQuerySkill;

#[async_trait]
impl Skill for PostgresQuerySkill {
    fn name(&self) -> &str {
        "postgres_query"
    }

    fn description(&self) -> &str {
        "Execute a SELECT query on a PostgreSQL database and return results as JSON"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "connection_string": {
                    "type": "string",
                    "description": "PostgreSQL connection string (e.g., 'host=localhost user=postgres password=secret dbname=mydb')"
                },
                "query": {
                    "type": "string",
                    "description": "SQL SELECT query to execute"
                }
            },
            "required": ["connection_string", "query"]
        })
    }

    async fn execute(&self, input: SkillInput) -> Result<SkillOutput, SkillError> {
        let connection_string = input.args.get("connection_string")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::Execution("Missing 'connection_string' parameter".to_string()))?;

        let query = input.args.get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::Execution("Missing 'query' parameter".to_string()))?;

        match postgres_query(connection_string, query).await {
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

/// PostgreSQL execute skill - executes INSERT/UPDATE/DELETE statements
pub struct PostgresExecuteSkill;

#[async_trait]
impl Skill for PostgresExecuteSkill {
    fn name(&self) -> &str {
        "postgres_execute"
    }

    fn description(&self) -> &str {
        "Execute an INSERT, UPDATE, or DELETE statement on a PostgreSQL database"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "connection_string": {
                    "type": "string",
                    "description": "PostgreSQL connection string (e.g., 'host=localhost user=postgres password=secret dbname=mydb')"
                },
                "statement": {
                    "type": "string",
                    "description": "SQL statement to execute"
                }
            },
            "required": ["connection_string", "statement"]
        })
    }

    async fn execute(&self, input: SkillInput) -> Result<SkillOutput, SkillError> {
        let connection_string = input.args.get("connection_string")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::Execution("Missing 'connection_string' parameter".to_string()))?;

        let statement = input.args.get("statement")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::Execution("Missing 'statement' parameter".to_string()))?;

        match postgres_execute(connection_string, statement).await {
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

/// Execute a SELECT query and return results as JSON
async fn postgres_query(connection_string: &str, query: &str) -> Result<Vec<serde_json::Value>, String> {
    let (client, connection) = tokio_postgres::connect(connection_string, NoTls).await
        .map_err(|e| format!("Failed to connect to PostgreSQL: {}", e))?;

    // Spawn connection in background
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("PostgreSQL connection error: {}", e);
        }
    });

    let rows = client.query(query, &[]).await
        .map_err(|e| format!("Failed to execute query: {}", e))?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row_to_json(&row));
    }

    Ok(result)
}

/// Execute an INSERT/UPDATE/DELETE statement
async fn postgres_execute(connection_string: &str, statement: &str) -> Result<u64, String> {
    let (client, connection) = tokio_postgres::connect(connection_string, NoTls).await
        .map_err(|e| format!("Failed to connect to PostgreSQL: {}", e))?;

    // Spawn connection in background
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("PostgreSQL connection error: {}", e);
        }
    });

    let rows_affected = client.execute(statement, &[]).await
        .map_err(|e| format!("Failed to execute statement: {}", e))?;

    Ok(rows_affected)
}

/// Convert a PostgreSQL row to JSON
fn row_to_json(row: &Row) -> serde_json::Value {
    let mut obj = serde_json::Map::new();

    for (idx, column) in row.columns().iter().enumerate() {
        let name = column.name();
        let value = match column.type_().name() {
            "int2" | "int4" => {
                row.try_get::<_, i32>(idx)
                    .map(|v| json!(v))
                    .unwrap_or(json!(null))
            }
            "int8" => {
                row.try_get::<_, i64>(idx)
                    .map(|v| json!(v))
                    .unwrap_or(json!(null))
            }
            "float4" | "float8" => {
                row.try_get::<_, f64>(idx)
                    .map(|v| json!(v))
                    .unwrap_or(json!(null))
            }
            "bool" => {
                row.try_get::<_, bool>(idx)
                    .map(|v| json!(v))
                    .unwrap_or(json!(null))
            }
            "text" | "varchar" | "char" | "name" => {
                row.try_get::<_, String>(idx)
                    .map(|v| json!(v))
                    .unwrap_or(json!(null))
            }
            _ => {
                // Try to get as string for other types
                row.try_get::<_, String>(idx)
                    .map(|v| json!(v))
                    .unwrap_or(json!(null))
            }
        };
        obj.insert(name.to_string(), value);
    }

    json!(obj)
}

/// Create PostgreSQL skills
pub fn create_postgres_skills() -> Vec<Arc<dyn Skill>> {
    vec![
        Arc::new(PostgresQuerySkill),
        Arc::new(PostgresExecuteSkill),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_postgres_query_connection_error() {
        let skill = PostgresQuerySkill;

        // Test with invalid connection string
        let input = SkillInput {
            name: "postgres_query".to_string(),
            args: json!({
                "connection_string": "host=invalid_host user=test",
                "query": "SELECT 1"
            }),
        };

        let output = skill.execute(input).await.unwrap();
        assert!(!output.success);
        assert!(output.error.is_some());
    }

    #[tokio::test]
    async fn test_postgres_execute_connection_error() {
        let skill = PostgresExecuteSkill;

        // Test with invalid connection string
        let input = SkillInput {
            name: "postgres_execute".to_string(),
            args: json!({
                "connection_string": "host=invalid_host user=test",
                "statement": "CREATE TABLE test (id INT)"
            }),
        };

        let output = skill.execute(input).await.unwrap();
        assert!(!output.success);
        assert!(output.error.is_some());
    }
}
