//! Compute skills for mathematical calculations and unit conversions

use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;

use crate::{Skill, SkillInput, SkillOutput, SkillError};

/// Calculate skill - evaluates mathematical expressions
pub struct CalculateSkill;

#[async_trait]
impl Skill for CalculateSkill {
    fn name(&self) -> &str {
        "calculate"
    }

    fn description(&self) -> &str {
        "Evaluate mathematical expressions. Supports basic operations (+, -, *, /), \
         parentheses, and common functions (sqrt, pow, sin, cos, tan, log, ln, abs, etc.)"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "Mathematical expression to evaluate (e.g., '2 + 2 * 3', 'sqrt(16)', 'pow(2, 8)')"
                }
            },
            "required": ["expression"]
        })
    }

    async fn execute(&self, input: SkillInput) -> Result<SkillOutput, SkillError> {
        let expression = input.args.get("expression")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::Execution("Missing 'expression' parameter".to_string()))?;

        match evaluate_expression(expression) {
            Ok(result) => Ok(SkillOutput {
                success: true,
                result: json!(result),
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

/// Convert units skill - converts between different units
pub struct ConvertUnitsSkill;

#[async_trait]
impl Skill for ConvertUnitsSkill {
    fn name(&self) -> &str {
        "convert_units"
    }

    fn description(&self) -> &str {
        "Convert between different units. Supports length (m, cm, km, ft, in, mi), \
         weight (kg, g, lb, oz), temperature (C, F, K), and time (s, min, h, day)"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "value": {
                    "type": "number",
                    "description": "The numeric value to convert"
                },
                "from_unit": {
                    "type": "string",
                    "description": "Source unit (e.g., 'cm', 'kg', 'F')"
                },
                "to_unit": {
                    "type": "string",
                    "description": "Target unit (e.g., 'm', 'lb', 'C')"
                }
            },
            "required": ["value", "from_unit", "to_unit"]
        })
    }

    async fn execute(&self, input: SkillInput) -> Result<SkillOutput, SkillError> {
        let value = input.args.get("value")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| SkillError::Execution("Missing or invalid 'value' parameter".to_string()))?;

        let from_unit = input.args.get("from_unit")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::Execution("Missing 'from_unit' parameter".to_string()))?;

        let to_unit = input.args.get("to_unit")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::Execution("Missing 'to_unit' parameter".to_string()))?;

        match convert_units(value, from_unit, to_unit) {
            Ok(result) => Ok(SkillOutput {
                success: true,
                result: json!({
                    "value": result,
                    "unit": to_unit
                }),
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

/// Evaluate a mathematical expression
fn evaluate_expression(expr: &str) -> Result<f64, String> {
    // Use meval crate for expression evaluation
    meval::eval_str(expr)
        .map_err(|e| format!("Failed to evaluate expression: {}", e))
}

/// Convert between units
fn convert_units(value: f64, from: &str, to: &str) -> Result<f64, String> {
    // Normalize unit names to lowercase
    let from = from.to_lowercase();
    let to = to.to_lowercase();

    // Check if units are in the same category
    let category = get_unit_category(&from)
        .ok_or_else(|| format!("Unknown unit: {}", from))?;

    let to_category = get_unit_category(&to)
        .ok_or_else(|| format!("Unknown unit: {}", to))?;

    if category != to_category {
        return Err(format!("Cannot convert between {} and {} (different categories)", from, to));
    }

    // Convert to base unit, then to target unit
    let base_value = to_base_unit(value, &from)?;
    from_base_unit(base_value, &to)
}

#[derive(Debug, PartialEq)]
enum UnitCategory {
    Length,
    Weight,
    Temperature,
    Time,
}

fn get_unit_category(unit: &str) -> Option<UnitCategory> {
    match unit {
        "m" | "cm" | "mm" | "km" | "ft" | "in" | "yd" | "mi" => Some(UnitCategory::Length),
        "kg" | "g" | "mg" | "lb" | "oz" | "ton" => Some(UnitCategory::Weight),
        "c" | "f" | "k" | "celsius" | "fahrenheit" | "kelvin" => Some(UnitCategory::Temperature),
        "s" | "ms" | "min" | "h" | "hour" | "day" | "week" => Some(UnitCategory::Time),
        _ => None,
    }
}

fn to_base_unit(value: f64, unit: &str) -> Result<f64, String> {
    match unit {
        // Length (base: meter)
        "m" => Ok(value),
        "cm" => Ok(value / 100.0),
        "mm" => Ok(value / 1000.0),
        "km" => Ok(value * 1000.0),
        "ft" => Ok(value * 0.3048),
        "in" => Ok(value * 0.0254),
        "yd" => Ok(value * 0.9144),
        "mi" => Ok(value * 1609.34),

        // Weight (base: kilogram)
        "kg" => Ok(value),
        "g" => Ok(value / 1000.0),
        "mg" => Ok(value / 1_000_000.0),
        "lb" => Ok(value * 0.453592),
        "oz" => Ok(value * 0.0283495),
        "ton" => Ok(value * 1000.0),

        // Temperature (base: Celsius)
        "c" | "celsius" => Ok(value),
        "f" | "fahrenheit" => Ok((value - 32.0) * 5.0 / 9.0),
        "k" | "kelvin" => Ok(value - 273.15),

        // Time (base: second)
        "s" => Ok(value),
        "ms" => Ok(value / 1000.0),
        "min" => Ok(value * 60.0),
        "h" | "hour" => Ok(value * 3600.0),
        "day" => Ok(value * 86400.0),
        "week" => Ok(value * 604800.0),

        _ => Err(format!("Unknown unit: {}", unit)),
    }
}

fn from_base_unit(value: f64, unit: &str) -> Result<f64, String> {
    match unit {
        // Length (base: meter)
        "m" => Ok(value),
        "cm" => Ok(value * 100.0),
        "mm" => Ok(value * 1000.0),
        "km" => Ok(value / 1000.0),
        "ft" => Ok(value / 0.3048),
        "in" => Ok(value / 0.0254),
        "yd" => Ok(value / 0.9144),
        "mi" => Ok(value / 1609.34),

        // Weight (base: kilogram)
        "kg" => Ok(value),
        "g" => Ok(value * 1000.0),
        "mg" => Ok(value * 1_000_000.0),
        "lb" => Ok(value / 0.453592),
        "oz" => Ok(value / 0.0283495),
        "ton" => Ok(value / 1000.0),

        // Temperature (base: Celsius)
        "c" | "celsius" => Ok(value),
        "f" | "fahrenheit" => Ok(value * 9.0 / 5.0 + 32.0),
        "k" | "kelvin" => Ok(value + 273.15),

        // Time (base: second)
        "s" => Ok(value),
        "ms" => Ok(value * 1000.0),
        "min" => Ok(value / 60.0),
        "h" | "hour" => Ok(value / 3600.0),
        "day" => Ok(value / 86400.0),
        "week" => Ok(value / 604800.0),

        _ => Err(format!("Unknown unit: {}", unit)),
    }
}

/// Create compute skills
pub fn create_compute_skills() -> Vec<Arc<dyn Skill>> {
    vec![
        Arc::new(CalculateSkill),
        Arc::new(ConvertUnitsSkill),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_calculate_basic() {
        let skill = CalculateSkill;
        let input = SkillInput {
            name: "calculate".to_string(),
            args: json!({"expression": "2 + 2"}),
        };
        let output = skill.execute(input).await.unwrap();
        assert!(output.success);
        assert_eq!(output.result.as_f64().unwrap(), 4.0);
    }

    #[tokio::test]
    async fn test_calculate_complex() {
        let skill = CalculateSkill;
        let input = SkillInput {
            name: "calculate".to_string(),
            args: json!({"expression": "sqrt(16) + pow(2, 3)"}),
        };
        let output = skill.execute(input).await.unwrap();
        assert!(output.success);
        assert_eq!(output.result.as_f64().unwrap(), 12.0);
    }

    #[tokio::test]
    async fn test_convert_length() {
        let skill = ConvertUnitsSkill;
        let input = SkillInput {
            name: "convert_units".to_string(),
            args: json!({"value": 100, "from_unit": "cm", "to_unit": "m"}),
        };
        let output = skill.execute(input).await.unwrap();
        assert!(output.success);
        assert_eq!(output.result["value"].as_f64().unwrap(), 1.0);
    }

    #[tokio::test]
    async fn test_convert_temperature() {
        let skill = ConvertUnitsSkill;
        let input = SkillInput {
            name: "convert_units".to_string(),
            args: json!({"value": 32, "from_unit": "F", "to_unit": "C"}),
        };
        let output = skill.execute(input).await.unwrap();
        assert!(output.success);
        assert!((output.result["value"].as_f64().unwrap() - 0.0).abs() < 0.01);
    }
}
