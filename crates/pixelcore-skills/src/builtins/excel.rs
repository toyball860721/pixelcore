//! Excel file processing skills

use async_trait::async_trait;
use calamine::{Reader, open_workbook, Xlsx, Data};
use rust_xlsxwriter::{Workbook, Format};
use serde_json::json;
use std::sync::Arc;

use crate::{Skill, SkillInput, SkillOutput, SkillError};

/// Excel read skill - reads Excel files and returns data as JSON
pub struct ExcelReadSkill;

#[async_trait]
impl Skill for ExcelReadSkill {
    fn name(&self) -> &str {
        "excel_read"
    }

    fn description(&self) -> &str {
        "Read an Excel file and return its contents as JSON"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the Excel file"
                },
                "sheet_name": {
                    "type": "string",
                    "description": "Sheet name to read (optional, defaults to first sheet)"
                }
            },
            "required": ["file_path"]
        })
    }

    async fn execute(&self, input: SkillInput) -> Result<SkillOutput, SkillError> {
        let file_path = input.args.get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::Execution("Missing 'file_path' parameter".to_string()))?;

        let sheet_name = input.args.get("sheet_name")
            .and_then(|v| v.as_str());

        match read_excel(file_path, sheet_name) {
            Ok(data) => Ok(SkillOutput {
                success: true,
                result: data,
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

/// Excel write skill - writes JSON data to an Excel file
pub struct ExcelWriteSkill;

#[async_trait]
impl Skill for ExcelWriteSkill {
    fn name(&self) -> &str {
        "excel_write"
    }

    fn description(&self) -> &str {
        "Write JSON data to an Excel file"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to save the Excel file"
                },
                "data": {
                    "type": "array",
                    "description": "Array of objects to write as rows"
                },
                "sheet_name": {
                    "type": "string",
                    "description": "Sheet name (optional, defaults to 'Sheet1')"
                }
            },
            "required": ["file_path", "data"]
        })
    }

    async fn execute(&self, input: SkillInput) -> Result<SkillOutput, SkillError> {
        let file_path = input.args.get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::Execution("Missing 'file_path' parameter".to_string()))?;

        let data = input.args.get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| SkillError::Execution("Missing 'data' parameter or not an array".to_string()))?;

        let sheet_name = input.args.get("sheet_name")
            .and_then(|v| v.as_str())
            .unwrap_or("Sheet1");

        match write_excel(file_path, data, sheet_name) {
            Ok(_) => Ok(SkillOutput {
                success: true,
                result: json!({"file_path": file_path, "rows": data.len()}),
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

/// Read Excel file and return data as JSON
fn read_excel(file_path: &str, sheet_name: Option<&str>) -> Result<serde_json::Value, String> {
    let mut workbook: Xlsx<_> = open_workbook(file_path)
        .map_err(|e| format!("Failed to open Excel file: {}", e))?;

    let sheet_name = if let Some(name) = sheet_name {
        name.to_string()
    } else {
        workbook.sheet_names().first()
            .ok_or_else(|| "No sheets found in workbook".to_string())?
            .clone()
    };

    let range = workbook.worksheet_range(&sheet_name)
        .map_err(|e| format!("Failed to read sheet '{}': {}", sheet_name, e))?;

    let mut rows = Vec::new();
    let mut headers: Vec<String> = Vec::new();

    for (row_idx, row) in range.rows().enumerate() {
        if row_idx == 0 {
            // First row is headers
            headers = row.iter()
                .map(|cell| cell.to_string())
                .collect();
        } else {
            // Data rows
            let mut obj = serde_json::Map::new();
            for (col_idx, cell) in row.iter().enumerate() {
                if col_idx < headers.len() {
                    let value = cell_to_json(cell);
                    obj.insert(headers[col_idx].clone(), value);
                }
            }
            rows.push(json!(obj));
        }
    }

    Ok(json!({
        "sheet_name": sheet_name,
        "rows": rows
    }))
}

/// Write JSON data to Excel file
fn write_excel(file_path: &str, data: &[serde_json::Value], sheet_name: &str) -> Result<(), String> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.set_name(sheet_name)
        .map_err(|e| format!("Failed to set sheet name: {}", e))?;

    if data.is_empty() {
        workbook.save(file_path)
            .map_err(|e| format!("Failed to save Excel file: {}", e))?;
        return Ok(());
    }

    // Extract headers from first object
    let headers: Vec<String> = if let Some(first_obj) = data.first().and_then(|v| v.as_object()) {
        first_obj.keys().cloned().collect()
    } else {
        return Err("Data must be an array of objects".to_string());
    };

    // Create header format
    let header_format = Format::new()
        .set_bold();

    // Write headers
    for (col_idx, header) in headers.iter().enumerate() {
        worksheet.write_string_with_format(0, col_idx as u16, header, &header_format)
            .map_err(|e| format!("Failed to write header: {}", e))?;
    }

    // Write data rows
    for (row_idx, row_data) in data.iter().enumerate() {
        if let Some(obj) = row_data.as_object() {
            for (col_idx, header) in headers.iter().enumerate() {
                if let Some(value) = obj.get(header) {
                    let row = (row_idx + 1) as u32;
                    let col = col_idx as u16;

                    match value {
                        serde_json::Value::String(s) => {
                            worksheet.write_string(row, col, s)
                                .map_err(|e| format!("Failed to write string: {}", e))?;
                        }
                        serde_json::Value::Number(n) => {
                            if let Some(i) = n.as_i64() {
                                worksheet.write_number(row, col, i as f64)
                                    .map_err(|e| format!("Failed to write number: {}", e))?;
                            } else if let Some(f) = n.as_f64() {
                                worksheet.write_number(row, col, f)
                                    .map_err(|e| format!("Failed to write number: {}", e))?;
                            }
                        }
                        serde_json::Value::Bool(b) => {
                            worksheet.write_boolean(row, col, *b)
                                .map_err(|e| format!("Failed to write boolean: {}", e))?;
                        }
                        serde_json::Value::Null => {
                            worksheet.write_blank(row, col, &Format::new())
                                .map_err(|e| format!("Failed to write blank: {}", e))?;
                        }
                        _ => {
                            worksheet.write_string(row, col, &value.to_string())
                                .map_err(|e| format!("Failed to write value: {}", e))?;
                        }
                    }
                }
            }
        }
    }

    workbook.save(file_path)
        .map_err(|e| format!("Failed to save Excel file: {}", e))?;

    Ok(())
}

/// Convert calamine Data to JSON value
fn cell_to_json(cell: &Data) -> serde_json::Value {
    match cell {
        Data::Int(i) => json!(i),
        Data::Float(f) => json!(f),
        Data::String(s) => json!(s),
        Data::Bool(b) => json!(b),
        Data::Empty => json!(null),
        Data::Error(e) => json!(format!("Error: {:?}", e)),
        Data::DateTime(dt) => json!(format!("{:?}", dt)),
        Data::DateTimeIso(dt) => json!(dt),
        Data::DurationIso(d) => json!(d),
    }
}

/// Create Excel skills
pub fn create_excel_skills() -> Vec<Arc<dyn Skill>> {
    vec![
        Arc::new(ExcelReadSkill),
        Arc::new(ExcelWriteSkill),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_excel_write_and_read() {
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_str().unwrap();

        // Write data
        let write_skill = ExcelWriteSkill;
        let data = vec![
            json!({"name": "Alice", "age": 30, "active": true}),
            json!({"name": "Bob", "age": 25, "active": false}),
            json!({"name": "Charlie", "age": 35, "active": true}),
        ];

        let write_input = SkillInput {
            name: "excel_write".to_string(),
            args: json!({
                "file_path": file_path,
                "data": data,
                "sheet_name": "Users"
            }),
        };

        let write_output = write_skill.execute(write_input).await.unwrap();
        assert!(write_output.success);
        assert_eq!(write_output.result["rows"], 3);

        // Read data back
        let read_skill = ExcelReadSkill;
        let read_input = SkillInput {
            name: "excel_read".to_string(),
            args: json!({
                "file_path": file_path,
                "sheet_name": "Users"
            }),
        };

        let read_output = read_skill.execute(read_input).await.unwrap();
        assert!(read_output.success);
        assert_eq!(read_output.result["sheet_name"], "Users");

        let rows = read_output.result["rows"].as_array().unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0]["name"], "Alice");
        assert_eq!(rows[1]["name"], "Bob");
        assert_eq!(rows[2]["name"], "Charlie");
    }
}
