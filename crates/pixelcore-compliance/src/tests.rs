use super::*;
use chrono::{Duration, Utc};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

// GDPR 测试
#[test]
fn test_gdpr_create_request() {
    let gdpr = GdprManager::new();
    let user_id = Uuid::new_v4();

    let request = gdpr
        .create_request(user_id, DataSubjectRight::Access)
        .unwrap();

    assert_eq!(request.user_id, user_id);
    assert_eq!(request.right, DataSubjectRight::Access);
    assert_eq!(request.status, RequestStatus::Pending);
}

#[test]
fn test_gdpr_complete_request() {
    let gdpr = GdprManager::new();
    let user_id = Uuid::new_v4();

    let request = gdpr
        .create_request(user_id, DataSubjectRight::Erasure)
        .unwrap();

    gdpr.complete_request(request.id).unwrap();

    let updated = gdpr.get_request(request.id).unwrap();
    assert_eq!(updated.status, RequestStatus::Completed);
    assert!(updated.completed_at.is_some());
}

#[test]
fn test_gdpr_consent_management() {
    let gdpr = GdprManager::new();
    let user_id = Uuid::new_v4();

    // 记录同意
    let consent = gdpr
        .record_consent(user_id, "Marketing".to_string(), "1.0".to_string())
        .unwrap();

    assert!(consent.is_active());

    // 撤回同意
    gdpr.withdraw_consent(consent.id).unwrap();

    let consents = gdpr.get_user_consents(user_id);
    assert_eq!(consents.len(), 1);
    assert!(!consents[0].is_active());
}

#[test]
fn test_gdpr_retention_policy() {
    let gdpr = GdprManager::new();

    let policy = RetentionPolicy::new(
        "user_logs".to_string(),
        365,
        "Keep user logs for 1 year".to_string(),
    );

    gdpr.add_retention_policy(policy.clone()).unwrap();

    let retrieved = gdpr.get_retention_policy("user_logs").unwrap();
    assert_eq!(retrieved.retention_period_days, 365);

    // 测试数据是否应该删除
    assert!(!gdpr.should_delete_data("user_logs", 300));
    assert!(gdpr.should_delete_data("user_logs", 400));
}

#[test]
fn test_gdpr_statistics() {
    let gdpr = GdprManager::new();
    let user_id = Uuid::new_v4();

    gdpr.create_request(user_id, DataSubjectRight::Access)
        .unwrap();
    gdpr.create_request(user_id, DataSubjectRight::Erasure)
        .unwrap();
    gdpr.record_consent(user_id, "Terms".to_string(), "1.0".to_string())
        .unwrap();

    let stats = gdpr.get_statistics();
    assert_eq!(stats.total_requests, 2);
    assert_eq!(stats.pending_requests, 2);
    assert_eq!(stats.total_consents, 1);
    assert_eq!(stats.active_consents, 1);
}

// 数据导出测试
#[test]
fn test_data_export_create_request() {
    let exporter = DataExporter::new();
    let user_id = Uuid::new_v4();

    let request = exporter
        .create_export_request(user_id, ExportFormat::Json)
        .unwrap();

    assert_eq!(request.user_id, user_id);
    assert_eq!(request.format, ExportFormat::Json);
}

#[test]
fn test_data_export_to_json() {
    let exporter = DataExporter::new();

    let user_data = UserData {
        user_id: Uuid::new_v4(),
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
        created_at: "2024-01-01".to_string(),
        profile: HashMap::new(),
        consents: vec![],
        activities: vec![],
    };

    let json = exporter.export_to_json(&user_data).unwrap();
    assert!(json.contains("test@example.com"));
    assert!(json.contains("Test User"));
}

#[test]
fn test_data_export_to_csv() {
    let exporter = DataExporter::new();

    let user_data = UserData {
        user_id: Uuid::new_v4(),
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
        created_at: "2024-01-01".to_string(),
        profile: HashMap::new(),
        consents: vec![],
        activities: vec![],
    };

    let csv = exporter.export_to_csv(&user_data).unwrap();
    assert!(csv.contains("test@example.com"));
    assert!(csv.contains("Test User"));
}

#[test]
fn test_data_export_execute() {
    let exporter = DataExporter::new();
    let user_id = Uuid::new_v4();

    let request = exporter
        .create_export_request(user_id, ExportFormat::Json)
        .unwrap();

    let user_data = UserData {
        user_id,
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
        created_at: "2024-01-01".to_string(),
        profile: HashMap::new(),
        consents: vec![],
        activities: vec![],
    };

    let exported = exporter.execute_export(request.id, &user_data).unwrap();
    assert!(exported.contains("test@example.com"));

    let updated_request = exporter.get_export_request(request.id).unwrap();
    assert!(updated_request.completed_at.is_some());
}

// 数据删除测试
#[test]
fn test_data_deletion_create_request() {
    let deleter = DataDeleter::new();
    let user_id = Uuid::new_v4();

    let request = deleter
        .create_deletion_request(user_id, DeletionType::Soft)
        .unwrap();

    assert_eq!(request.user_id, user_id);
    assert_eq!(request.deletion_type, DeletionType::Soft);
}

#[test]
fn test_data_deletion_soft() {
    let deleter = DataDeleter::new();
    let user_id = Uuid::new_v4();

    let request = deleter
        .create_deletion_request(user_id, DeletionType::Soft)
        .unwrap();

    let deleted = deleter.execute_soft_deletion(request.id).unwrap();
    assert!(deleted.len() > 0);
    assert!(deleter.is_soft_deleted(user_id));
}

#[test]
fn test_data_deletion_hard() {
    let deleter = DataDeleter::new();
    let user_id = Uuid::new_v4();

    let request = deleter
        .create_deletion_request(user_id, DeletionType::Hard)
        .unwrap();

    let deleted = deleter.execute_hard_deletion(request.id).unwrap();
    assert!(deleted.len() > 0);
    assert!(deleted.contains(&format!("user:{}", user_id)));
}

#[test]
fn test_data_deletion_restore() {
    let deleter = DataDeleter::new();
    let user_id = Uuid::new_v4();

    let request = deleter
        .create_deletion_request(user_id, DeletionType::Soft)
        .unwrap();

    deleter.execute_soft_deletion(request.id).unwrap();
    assert!(deleter.is_soft_deleted(user_id));

    deleter.restore_soft_deleted_user(user_id).unwrap();
    assert!(!deleter.is_soft_deleted(user_id));
}

#[test]
fn test_data_deletion_statistics() {
    let deleter = DataDeleter::new();
    let user1 = Uuid::new_v4();
    let user2 = Uuid::new_v4();

    let req1 = deleter
        .create_deletion_request(user1, DeletionType::Soft)
        .unwrap();
    let req2 = deleter
        .create_deletion_request(user2, DeletionType::Hard)
        .unwrap();

    deleter.execute_soft_deletion(req1.id).unwrap();
    deleter.execute_hard_deletion(req2.id).unwrap();

    let stats = deleter.get_statistics();
    assert_eq!(stats.total_requests, 2);
    assert_eq!(stats.completed_requests, 2);
    assert_eq!(stats.soft_deletions, 1);
    assert_eq!(stats.hard_deletions, 1);
}

// 不可篡改审计日志测试
#[test]
fn test_immutable_audit_log() {
    let logger = ImmutableAuditLogger::new(100);
    let user_id = Uuid::new_v4();

    let log = logger.log(
        Some(user_id),
        "CREATE".to_string(),
        "User".to_string(),
        Some(user_id),
        json!({"action": "user_created"}),
    );

    assert!(log.verify());
    assert_eq!(logger.count(), 1);
}

#[test]
fn test_immutable_audit_chain() {
    let logger = ImmutableAuditLogger::new(100);
    let user_id = Uuid::new_v4();

    // 记录多条日志
    for i in 0..5 {
        logger.log(
            Some(user_id),
            format!("ACTION_{}", i),
            "Resource".to_string(),
            None,
            json!({"index": i}),
        );
    }

    // 验证链的完整性
    assert!(logger.verify_chain().is_ok());
    assert_eq!(logger.count(), 5);
}

#[test]
fn test_immutable_audit_search() {
    let logger = ImmutableAuditLogger::new(100);
    let user1 = Uuid::new_v4();
    let user2 = Uuid::new_v4();

    logger.log(
        Some(user1),
        "CREATE".to_string(),
        "User".to_string(),
        None,
        json!({}),
    );

    logger.log(
        Some(user2),
        "UPDATE".to_string(),
        "User".to_string(),
        None,
        json!({}),
    );

    let user1_logs = logger.get_user_logs(user1);
    assert_eq!(user1_logs.len(), 1);

    let create_logs = logger.search_logs(|log| log.action == "CREATE");
    assert_eq!(create_logs.len(), 1);
}

#[test]
fn test_immutable_audit_time_range() {
    let logger = ImmutableAuditLogger::new(100);
    let user_id = Uuid::new_v4();

    let now = Utc::now();

    logger.log(
        Some(user_id),
        "ACTION".to_string(),
        "Resource".to_string(),
        None,
        json!({}),
    );

    let logs = logger.get_logs_in_range(now - Duration::minutes(1), now + Duration::minutes(1));
    assert_eq!(logs.len(), 1);
}

#[test]
fn test_immutable_audit_statistics() {
    let logger = ImmutableAuditLogger::new(100);
    let user1 = Uuid::new_v4();
    let user2 = Uuid::new_v4();

    logger.log(
        Some(user1),
        "CREATE".to_string(),
        "User".to_string(),
        None,
        json!({}),
    );

    logger.log(
        Some(user2),
        "CREATE".to_string(),
        "User".to_string(),
        None,
        json!({}),
    );

    logger.log(
        Some(user1),
        "UPDATE".to_string(),
        "User".to_string(),
        None,
        json!({}),
    );

    let stats = logger.get_statistics();
    assert_eq!(stats.total_logs, 3);
    assert_eq!(stats.unique_users, 2);
}

// 合规报告测试
#[test]
fn test_compliance_reporter() {
    let reporter = ComplianceReporter::new();

    let start = Utc::now() - Duration::days(30);
    let end = Utc::now();

    let report = reporter.generate_gdpr_report(
        start,
        end,
        json!({
            "total_requests": 10,
            "completed": 8,
        }),
    );

    assert_eq!(report.report_type, ComplianceReportType::Gdpr);

    let reports = reporter.get_all_reports();
    assert_eq!(reports.len(), 1);
}

#[test]
fn test_compliance_reporter_by_type() {
    let reporter = ComplianceReporter::new();

    let start = Utc::now() - Duration::days(30);
    let end = Utc::now();

    reporter.generate_gdpr_report(start, end, json!({}));
    reporter.generate_data_processing_report(start, end, json!({}));
    reporter.generate_data_subject_requests_report(start, end, json!({}));

    let gdpr_reports = reporter.get_reports_by_type(ComplianceReportType::Gdpr);
    assert_eq!(gdpr_reports.len(), 1);

    let all_reports = reporter.get_all_reports();
    assert_eq!(all_reports.len(), 3);
}
