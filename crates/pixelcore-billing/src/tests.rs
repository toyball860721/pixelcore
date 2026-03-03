use super::*;
use chrono::{Duration, Utc, Datelike};
use uuid::Uuid;

#[tokio::test]
async fn test_usage_tracking() {
    let tracker = UsageTracker::new();
    let user_id = Uuid::new_v4();

    // 记录 API 调用
    let record = tracker
        .record_usage(user_id, UsageType::ApiCall, 10.0, "calls".to_string())
        .await
        .unwrap();

    assert_eq!(record.usage_type, UsageType::ApiCall);
    assert_eq!(record.quantity, 10.0);
}

#[tokio::test]
async fn test_usage_stats() {
    let tracker = UsageTracker::new();
    let user_id = Uuid::new_v4();

    // 记录多次使用
    tracker
        .record_usage(user_id, UsageType::ApiCall, 10.0, "calls".to_string())
        .await
        .unwrap();
    tracker
        .record_usage(user_id, UsageType::ApiCall, 5.0, "calls".to_string())
        .await
        .unwrap();
    tracker
        .record_usage(user_id, UsageType::ComputeHours, 2.0, "hours".to_string())
        .await
        .unwrap();

    // 获取统计
    let period_start = Utc::now() - Duration::hours(1);
    let period_end = Utc::now() + Duration::hours(1);

    let stats = tracker
        .get_usage_stats(user_id, period_start, period_end)
        .await
        .unwrap();

    assert_eq!(stats.usage_by_type.get(&UsageType::ApiCall), Some(&15.0));
    assert_eq!(stats.usage_by_type.get(&UsageType::ComputeHours), Some(&2.0));
    assert_eq!(stats.total_usage, 17.0);
}

#[tokio::test]
async fn test_quota_management() {
    let tracker = UsageTracker::new();
    let user_id = Uuid::new_v4();

    // 设置配额
    let quota = tracker
        .set_quota(user_id, UsageType::ApiCall, 100.0, 30)
        .await
        .unwrap();

    assert_eq!(quota.limit, 100.0);
    assert_eq!(quota.used, 0.0);

    // 记录使用量
    tracker
        .record_usage(user_id, UsageType::ApiCall, 30.0, "calls".to_string())
        .await
        .unwrap();

    // 检查配额
    let updated_quota = tracker.get_quota(user_id, UsageType::ApiCall).await.unwrap();
    assert_eq!(updated_quota.used, 30.0);
    assert_eq!(updated_quota.remaining(), 70.0);
}

#[tokio::test]
async fn test_quota_exceeded() {
    let tracker = UsageTracker::new();
    let user_id = Uuid::new_v4();

    // 设置配额
    tracker
        .set_quota(user_id, UsageType::ApiCall, 50.0, 30)
        .await
        .unwrap();

    // 记录使用量
    tracker
        .record_usage(user_id, UsageType::ApiCall, 30.0, "calls".to_string())
        .await
        .unwrap();

    // 尝试超出配额
    let result = tracker
        .record_usage(user_id, UsageType::ApiCall, 30.0, "calls".to_string())
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_pay_as_you_go_pricing() {
    let rule = BillingRule::new(
        "API Calls".to_string(),
        UsageType::ApiCall,
        PricingModel::PayAsYouGo {
            unit_price: 0.01,
            unit: "call".to_string(),
        },
    );

    let cost = rule.calculate_cost(100.0);
    assert_eq!(cost, 1.0); // 100 calls * $0.01
}

#[tokio::test]
async fn test_subscription_pricing() {
    let rule = BillingRule::new(
        "API Subscription".to_string(),
        UsageType::ApiCall,
        PricingModel::Subscription {
            monthly_fee: 10.0,
            included_quota: 1000.0,
            overage_price: 0.02,
        },
    );

    // 在配额内
    let cost1 = rule.calculate_cost(500.0);
    assert_eq!(cost1, 10.0);

    // 超出配额
    let cost2 = rule.calculate_cost(1200.0);
    assert_eq!(cost2, 14.0); // $10 + 200 * $0.02
}

#[tokio::test]
async fn test_tiered_pricing() {
    let rule = BillingRule::new(
        "Storage".to_string(),
        UsageType::Storage,
        PricingModel::Tiered {
            tiers: vec![
                (100.0, 0.10),  // First 100 GB at $0.10/GB
                (400.0, 0.08),  // Next 400 GB at $0.08/GB
                (f64::MAX, 0.05), // Rest at $0.05/GB
            ],
        },
    );

    // 150 GB: 100*0.10 + 50*0.08 = 10 + 4 = 14
    let cost = rule.calculate_cost(150.0);
    assert_eq!(cost, 14.0);
}

#[tokio::test]
async fn test_billing_engine_add_rule() {
    let tracker = UsageTracker::new();
    let engine = BillingEngine::new(tracker);

    let rule = BillingRule::new(
        "API Calls".to_string(),
        UsageType::ApiCall,
        PricingModel::PayAsYouGo {
            unit_price: 0.01,
            unit: "call".to_string(),
        },
    );

    engine.add_rule(rule).await.unwrap();

    let retrieved_rule = engine.get_rule(&UsageType::ApiCall).await;
    assert!(retrieved_rule.is_some());
}

#[tokio::test]
async fn test_invoice_generation() {
    let tracker = UsageTracker::new();
    let engine = BillingEngine::new(tracker.clone());

    let user_id = Uuid::new_v4();

    // 添加计费规则
    let rule = BillingRule::new(
        "API Calls".to_string(),
        UsageType::ApiCall,
        PricingModel::PayAsYouGo {
            unit_price: 0.01,
            unit: "call".to_string(),
        },
    );
    engine.add_rule(rule).await.unwrap();

    // 记录使用量
    tracker
        .record_usage(user_id, UsageType::ApiCall, 100.0, "calls".to_string())
        .await
        .unwrap();

    // 生成账单
    let period_start = Utc::now() - Duration::hours(1);
    let period_end = Utc::now() + Duration::hours(1);

    let invoice = engine
        .generate_invoice(user_id, period_start, period_end)
        .await
        .unwrap();

    assert_eq!(invoice.user_id, user_id);
    assert_eq!(invoice.status, InvoiceStatus::Pending);
    assert_eq!(invoice.items.len(), 1);
    assert_eq!(invoice.subtotal, 1.0); // 100 calls * $0.01
}

#[tokio::test]
async fn test_invoice_payment() {
    let tracker = UsageTracker::new();
    let engine = BillingEngine::new(tracker.clone());

    let user_id = Uuid::new_v4();

    // 添加计费规则
    let rule = BillingRule::new(
        "API Calls".to_string(),
        UsageType::ApiCall,
        PricingModel::PayAsYouGo {
            unit_price: 0.01,
            unit: "call".to_string(),
        },
    );
    engine.add_rule(rule).await.unwrap();

    // 记录使用量
    tracker
        .record_usage(user_id, UsageType::ApiCall, 50.0, "calls".to_string())
        .await
        .unwrap();

    // 生成账单
    let period_start = Utc::now() - Duration::hours(1);
    let period_end = Utc::now() + Duration::hours(1);

    let invoice = engine
        .generate_invoice(user_id, period_start, period_end)
        .await
        .unwrap();

    // 标记为已支付
    let paid_invoice = engine.mark_invoice_paid(invoice.id).await.unwrap();

    assert_eq!(paid_invoice.status, InvoiceStatus::Paid);
    assert!(paid_invoice.paid_at.is_some());
}

#[tokio::test]
async fn test_cost_estimation() {
    let tracker = UsageTracker::new();
    let engine = BillingEngine::new(tracker);

    // 添加计费规则
    let rule = BillingRule::new(
        "Compute".to_string(),
        UsageType::ComputeHours,
        PricingModel::PayAsYouGo {
            unit_price: 0.50,
            unit: "hour".to_string(),
        },
    );
    engine.add_rule(rule).await.unwrap();

    // 预估费用
    let estimated_cost = engine
        .estimate_cost(UsageType::ComputeHours, 10.0)
        .await
        .unwrap();

    assert_eq!(estimated_cost, 5.0); // 10 hours * $0.50
}

#[tokio::test]
async fn test_monthly_invoice_generation() {
    let tracker = UsageTracker::new();
    let engine = BillingEngine::new(tracker.clone());

    let user_id = Uuid::new_v4();

    // 添加计费规则
    let rule = BillingRule::new(
        "API Calls".to_string(),
        UsageType::ApiCall,
        PricingModel::PayAsYouGo {
            unit_price: 0.01,
            unit: "call".to_string(),
        },
    );
    engine.add_rule(rule).await.unwrap();

    // 记录使用量
    tracker
        .record_usage(user_id, UsageType::ApiCall, 200.0, "calls".to_string())
        .await
        .unwrap();

    // 生成月度账单
    let now = Utc::now();
    let invoice = engine
        .generate_monthly_invoice(user_id, now.year(), now.month())
        .await
        .unwrap();

    assert_eq!(invoice.user_id, user_id);
    assert!(invoice.invoice_number.starts_with("INV-"));
}

#[tokio::test]
async fn test_quota_reset() {
    let tracker = UsageTracker::new();
    let user_id = Uuid::new_v4();

    // 设置配额
    tracker
        .set_quota(user_id, UsageType::ApiCall, 100.0, 30)
        .await
        .unwrap();

    // 记录使用量
    tracker
        .record_usage(user_id, UsageType::ApiCall, 50.0, "calls".to_string())
        .await
        .unwrap();

    // 重置配额
    tracker.reset_quota(user_id, UsageType::ApiCall).await.unwrap();

    // 检查配额已重置
    let quota = tracker.get_quota(user_id, UsageType::ApiCall).await.unwrap();
    assert_eq!(quota.used, 0.0);
}
