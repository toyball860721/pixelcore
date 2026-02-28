use anyhow::Result;
use pixelcore_skills::{Skill, SkillInput, create_compute_skills, create_data_skills};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                                                              ║");
    println!("║           新增 Skills 测试示例                                ║");
    println!("║                                                              ║");
    println!("║  测试计算 Skills 和数据处理 Skills                            ║");
    println!("║                                                              ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // 创建 Skills
    let compute_skills = create_compute_skills();
    let data_skills = create_data_skills();

    println!("═══ 第一部分：计算 Skills 测试 ═══\n");

    // 测试 calculate skill
    println!("【测试 1】基础计算");
    let calculate_skill = &compute_skills[0];
    let input = SkillInput {
        name: "calculate".to_string(),
        args: json!({"expression": "2 + 2 * 3"}),
    };
    let output = calculate_skill.execute(input).await?;
    println!("  表达式: 2 + 2 * 3");
    println!("  结果: {}", output.result);
    println!("  状态: {}\n", if output.success { "✅ 成功" } else { "❌ 失败" });

    // 测试复杂计算
    println!("【测试 2】复杂计算");
    let input = SkillInput {
        name: "calculate".to_string(),
        args: json!({"expression": "sqrt(16) + pow(2, 3)"}),
    };
    let output = calculate_skill.execute(input).await?;
    println!("  表达式: sqrt(16) + pow(2, 3)");
    println!("  结果: {}", output.result);
    println!("  状态: {}\n", if output.success { "✅ 成功" } else { "❌ 失败" });

    // 测试 convert_units skill
    println!("【测试 3】单位转换 - 长度");
    let convert_skill = &compute_skills[1];
    let input = SkillInput {
        name: "convert_units".to_string(),
        args: json!({"value": 100, "from_unit": "cm", "to_unit": "m"}),
    };
    let output = convert_skill.execute(input).await?;
    println!("  转换: 100 cm -> m");
    println!("  结果: {}", output.result);
    println!("  状态: {}\n", if output.success { "✅ 成功" } else { "❌ 失败" });

    // 测试温度转换
    println!("【测试 4】单位转换 - 温度");
    let input = SkillInput {
        name: "convert_units".to_string(),
        args: json!({"value": 32, "from_unit": "F", "to_unit": "C"}),
    };
    let output = convert_skill.execute(input).await?;
    println!("  转换: 32 F -> C");
    println!("  结果: {}", output.result);
    println!("  状态: {}\n", if output.success { "✅ 成功" } else { "❌ 失败" });

    println!("\n═══ 第二部分：数据处理 Skills 测试 ═══\n");

    // 测试 json_parse skill
    println!("【测试 5】JSON 解析");
    let json_parse_skill = &data_skills[0];
    let input = SkillInput {
        name: "json_parse".to_string(),
        args: json!({"json_string": r#"{"name": "Alice", "age": 30, "city": "Beijing"}"#}),
    };
    let output = json_parse_skill.execute(input).await?;
    println!("  输入: {{\"name\": \"Alice\", \"age\": 30, \"city\": \"Beijing\"}}");
    println!("  结果: {}", output.result);
    println!("  状态: {}\n", if output.success { "✅ 成功" } else { "❌ 失败" });

    // 测试 json_query skill
    println!("【测试 6】JSON 查询");
    let json_query_skill = &data_skills[1];
    let data = json!({
        "users": [
            {"name": "Alice", "age": 30, "skills": ["Rust", "Python"]},
            {"name": "Bob", "age": 25, "skills": ["JavaScript", "Go"]}
        ],
        "total": 2
    });
    let input = SkillInput {
        name: "json_query".to_string(),
        args: json!({"data": data, "path": "users.0.name"}),
    };
    let output = json_query_skill.execute(input).await?;
    println!("  查询路径: users.0.name");
    println!("  结果: {}", output.result);
    println!("  状态: {}\n", if output.success { "✅ 成功" } else { "❌ 失败" });

    // 测试查询数组
    let input = SkillInput {
        name: "json_query".to_string(),
        args: json!({"data": data, "path": "users.1.skills"}),
    };
    let output = json_query_skill.execute(input).await?;
    println!("  查询路径: users.1.skills");
    println!("  结果: {}", output.result);
    println!("  状态: {}\n", if output.success { "✅ 成功" } else { "❌ 失败" });

    // 测试 csv_parse skill
    println!("【测试 7】CSV 解析");
    let csv_parse_skill = &data_skills[2];
    let input = SkillInput {
        name: "csv_parse".to_string(),
        args: json!({"csv_string": "name,age,city\nAlice,30,Beijing\nBob,25,Shanghai\nCarol,28,Shenzhen"}),
    };
    let output = csv_parse_skill.execute(input).await?;
    println!("  输入 CSV:");
    println!("    name,age,city");
    println!("    Alice,30,Beijing");
    println!("    Bob,25,Shanghai");
    println!("    Carol,28,Shenzhen");
    println!("  结果: {}", serde_json::to_string_pretty(&output.result)?);
    println!("  状态: {}\n", if output.success { "✅ 成功" } else { "❌ 失败" });

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  测试完成！                                                   ║");
    println!("║                                                              ║");
    println!("║  新增 Skills:                                                ║");
    println!("║  - 计算 Skills: 2 个 (calculate, convert_units)             ║");
    println!("║  - 数据处理 Skills: 3 个 (json_parse, json_query, csv_parse)║");
    println!("║  - 总计: 5 个新 Skills                                       ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    Ok(())
}
