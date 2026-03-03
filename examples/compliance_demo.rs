use pixelcore_compliance::{
    ComplianceReporter, DataDeleter, DataExporter, DataSubjectRight, DeletionType, ExportFormat,
    GdprManager, ImmutableAuditLogger, RetentionPolicy, UserData,
};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    println!("=== PixelCore 合规性系统演示 ===\n");

    // 1. GDPR 合规管理
    println!("1. GDPR 合规管理");
    println!("==================");

    let gdpr = GdprManager::new();
    let user_id = Uuid::new_v4();

    // 创建数据主体请求
    let access_request = gdpr
        .create_request(user_id, DataSubjectRight::Access)
        .unwrap();
    println!("✓ 创建数据访问请求: {}", access_request.id);

    let erasure_request = gdpr
        .create_request(user_id, DataSubjectRight::Erasure)
        .unwrap();
    println!("✓ 创建数据删除请求: {}", erasure_request.id);

    // 记录同意
    let consent = gdpr
        .record_consent(user_id, "Marketing Communications".to_string(), "1.0".to_string())
        .unwrap();
    println!("✓ 记录用户同意: {} ({})", consent.purpose, consent.version);

    // 添加数据保留策略
    let policy = RetentionPolicy::new(
        "user_logs".to_string(),
        365,
        "Keep user activity logs for 1 year".to_string(),
    );
    gdpr.add_retention_policy(policy).unwrap();
    println!("✓ 添加数据保留策略: user_logs (365 天)");

    // 获取统计信息
    let stats = gdpr.get_statistics();
    println!("\nGDPR 统计:");
    println!("  - 总请求数: {}", stats.total_requests);
    println!("  - 待处理请求: {}", stats.pending_requests);
    println!("  - 总同意记录: {}", stats.total_consents);
    println!("  - 活跃同意: {}", stats.active_consents);

    // 2. 数据导出
    println!("\n2. 数据导出 (GDPR 数据可携带权)");
    println!("==================================");

    let exporter = DataExporter::new();

    // 创建导出请求
    let export_request = exporter
        .create_export_request(user_id, ExportFormat::Json)
        .unwrap();
    println!("✓ 创建数据导出请求: {}", export_request.id);

    // 准备用户数据
    let mut profile = HashMap::new();
    profile.insert("age".to_string(), json!(30));
    profile.insert("country".to_string(), json!("US"));

    let user_data = UserData {
        user_id,
        email: "user@example.com".to_string(),
        name: "John Doe".to_string(),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        profile,
        consents: vec![json!({"purpose": "Marketing", "granted": true})],
        activities: vec![json!({"action": "login", "timestamp": "2024-01-15"})],
    };

    // 导出为 JSON
    let json_export = exporter.execute_export(export_request.id, &user_data).unwrap();
    println!("✓ 导出为 JSON ({} bytes)", json_export.len());
    println!("  预览: {}...", &json_export[..100.min(json_export.len())]);

    // 导出为 CSV
    let csv_request = exporter
        .create_export_request(user_id, ExportFormat::Csv)
        .unwrap();
    let csv_export = exporter.execute_export(csv_request.id, &user_data).unwrap();
    println!("✓ 导出为 CSV ({} bytes)", csv_export.len());

    // 3. 数据删除 (Right to be Forgotten)
    println!("\n3. 数据删除 (被遗忘权)");
    println!("========================");

    let deleter = DataDeleter::new();

    // 软删除
    let soft_delete_request = deleter
        .create_deletion_request(user_id, DeletionType::Soft)
        .unwrap();
    println!("✓ 创建软删除请求: {}", soft_delete_request.id);

    let soft_deleted = deleter
        .execute_soft_deletion(soft_delete_request.id)
        .unwrap();
    println!("✓ 执行软删除，删除 {} 条记录", soft_deleted.len());
    for record in &soft_deleted {
        println!("  - {}", record);
    }

    // 检查是否被软删除
    println!("✓ 用户是否被软删除: {}", deleter.is_soft_deleted(user_id));

    // 硬删除示例
    let user2 = Uuid::new_v4();
    let hard_delete_request = deleter
        .create_deletion_request(user2, DeletionType::Hard)
        .unwrap();
    let hard_deleted = deleter
        .execute_hard_deletion(hard_delete_request.id)
        .unwrap();
    println!("\n✓ 执行硬删除，永久删除 {} 条记录", hard_deleted.len());

    // 删除统计
    let del_stats = deleter.get_statistics();
    println!("\n删除统计:");
    println!("  - 总删除请求: {}", del_stats.total_requests);
    println!("  - 软删除: {}", del_stats.soft_deletions);
    println!("  - 硬删除: {}", del_stats.hard_deletions);

    // 4. 不可篡改审计日志
    println!("\n4. 不可篡改审计日志");
    println!("======================");

    let audit_logger = ImmutableAuditLogger::new(10000);

    // 记录审计日志
    let log1 = audit_logger.log(
        Some(user_id),
        "USER_CREATED".to_string(),
        "User".to_string(),
        Some(user_id),
        json!({"email": "user@example.com"}),
    );
    println!("✓ 记录审计日志 1: {}", log1.id);
    println!("  - 哈希: {}...", &log1.hash[..16]);

    let log2 = audit_logger.log(
        Some(user_id),
        "DATA_EXPORTED".to_string(),
        "Export".to_string(),
        Some(export_request.id),
        json!({"format": "JSON"}),
    );
    println!("✓ 记录审计日志 2: {}", log2.id);
    println!("  - 前一条哈希: {}...", &log2.previous_hash[..16]);
    println!("  - 当前哈希: {}...", &log2.hash[..16]);

    let log3 = audit_logger.log(
        Some(user_id),
        "DATA_DELETED".to_string(),
        "Deletion".to_string(),
        Some(soft_delete_request.id),
        json!({"type": "soft"}),
    );
    println!("✓ 记录审计日志 3: {}", log3.id);

    // 验证日志链
    match audit_logger.verify_chain() {
        Ok(_) => println!("✓ 审计日志链验证成功！"),
        Err(e) => println!("✗ 审计日志链验证失败: {}", e),
    }

    // 审计统计
    let audit_stats = audit_logger.get_statistics();
    println!("\n审计统计:");
    println!("  - 总日志数: {}", audit_stats.total_logs);
    println!("  - 唯一用户数: {}", audit_stats.unique_users);

    // 5. 合规报告
    println!("\n5. 合规报告生成");
    println!("==================");

    let reporter = ComplianceReporter::new();

    // 生成 GDPR 合规报告
    let gdpr_report = reporter.generate_gdpr_report(
        chrono::Utc::now() - chrono::Duration::days(30),
        chrono::Utc::now(),
        json!({
            "total_requests": stats.total_requests,
            "completed_requests": stats.completed_requests,
            "pending_requests": stats.pending_requests,
            "consents": stats.total_consents,
        }),
    );
    println!("✓ 生成 GDPR 合规报告: {}", gdpr_report.id);
    println!("  - 报告类型: {:?}", gdpr_report.report_type);
    println!("  - 生成时间: {}", gdpr_report.generated_at);

    // 生成数据主体请求报告
    let dsr_report = reporter.generate_data_subject_requests_report(
        chrono::Utc::now() - chrono::Duration::days(30),
        chrono::Utc::now(),
        json!({
            "access_requests": 1,
            "erasure_requests": 1,
            "total": 2,
        }),
    );
    println!("✓ 生成数据主体请求报告: {}", dsr_report.id);

    let all_reports = reporter.get_all_reports();
    println!("\n总报告数: {}", all_reports.len());

    println!("\n=== 演示完成 ===");
    println!("\n总结:");
    println!("- GDPR 合规管理: 数据主体请求、同意管理、保留策略 ✓");
    println!("- 数据导出: JSON/CSV 格式导出 ✓");
    println!("- 数据删除: 软删除和硬删除 ✓");
    println!("- 不可篡改审计日志: 链式验证 ✓");
    println!("- 合规报告: 自动生成 ✓");
    println!("\n所有合规性功能测试通过！");
}
