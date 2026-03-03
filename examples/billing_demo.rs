use pixelcore_billing::{
    UsageTracker, UsageType, BillingEngine, BillingRule, PricingModel,
};
use uuid::Uuid;
use chrono::{Duration, Utc, Datelike};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Billing System Demo ===\n");

    // 创建使用量追踪器
    let tracker = UsageTracker::new();

    // Demo 1: 记录使用量
    println!("--- Demo 1: Usage Tracking ---");
    let user_id = Uuid::new_v4();
    println!("User ID: {}\n", user_id);

    // 记录 API 调用
    tracker
        .record_usage(user_id, UsageType::ApiCall, 150.0, "calls".to_string())
        .await?;
    println!("✓ Recorded 150 API calls");

    // 记录计算资源使用
    tracker
        .record_usage(user_id, UsageType::ComputeHours, 5.0, "hours".to_string())
        .await?;
    println!("✓ Recorded 5 compute hours");

    // 记录存储使用
    tracker
        .record_usage(user_id, UsageType::Storage, 100.0, "GB".to_string())
        .await?;
    println!("✓ Recorded 100 GB storage\n");

    // Demo 2: 使用量统计
    println!("--- Demo 2: Usage Statistics ---");
    let period_start = Utc::now() - Duration::hours(1);
    let period_end = Utc::now() + Duration::hours(1);

    let stats = tracker
        .get_usage_stats(user_id, period_start, period_end)
        .await?;

    println!("Usage Summary:");
    for (usage_type, quantity) in &stats.usage_by_type {
        println!("  {:?}: {}", usage_type, quantity);
    }
    println!("Total Usage: {}\n", stats.total_usage);

    // Demo 3: 配额管理
    println!("--- Demo 3: Quota Management ---");
    let user2_id = Uuid::new_v4();

    // 设置 API 调用配额
    let quota = tracker
        .set_quota(user2_id, UsageType::ApiCall, 1000.0, 30)
        .await?;

    println!("API Call Quota:");
    println!("  Limit: {}", quota.limit);
    println!("  Used: {}", quota.used);
    println!("  Remaining: {}", quota.remaining());

    // 记录使用量
    tracker
        .record_usage(user2_id, UsageType::ApiCall, 300.0, "calls".to_string())
        .await?;

    let updated_quota = tracker.get_quota(user2_id, UsageType::ApiCall).await.unwrap();
    println!("\nAfter usage:");
    println!("  Used: {}", updated_quota.used);
    println!("  Remaining: {}\n", updated_quota.remaining());

    // Demo 4: 计费规则
    println!("--- Demo 4: Billing Rules ---");
    let engine = BillingEngine::new(tracker.clone());

    // 按量计费规则
    let api_rule = BillingRule::new(
        "API Calls".to_string(),
        UsageType::ApiCall,
        PricingModel::PayAsYouGo {
            unit_price: 0.01,
            unit: "call".to_string(),
        },
    );
    engine.add_rule(api_rule).await?;
    println!("✓ Added Pay-as-you-go rule for API calls ($0.01/call)");

    // 订阅计费规则
    let compute_rule = BillingRule::new(
        "Compute Hours".to_string(),
        UsageType::ComputeHours,
        PricingModel::Subscription {
            monthly_fee: 50.0,
            included_quota: 10.0,
            overage_price: 8.0,
        },
    );
    engine.add_rule(compute_rule).await?;
    println!("✓ Added Subscription rule for compute ($50/month, 10 hours included)");

    // 阶梯定价规则
    let storage_rule = BillingRule::new(
        "Storage".to_string(),
        UsageType::Storage,
        PricingModel::Tiered {
            tiers: vec![
                (100.0, 0.10),    // First 100 GB at $0.10/GB
                (400.0, 0.08),    // Next 400 GB at $0.08/GB
                (f64::MAX, 0.05), // Rest at $0.05/GB
            ],
        },
    );
    engine.add_rule(storage_rule).await?;
    println!("✓ Added Tiered pricing rule for storage\n");

    // Demo 5: 费用预估
    println!("--- Demo 5: Cost Estimation ---");

    let api_cost = engine.estimate_cost(UsageType::ApiCall, 500.0).await?;
    println!("500 API calls would cost: ${:.2}", api_cost);

    let compute_cost = engine.estimate_cost(UsageType::ComputeHours, 15.0).await?;
    println!("15 compute hours would cost: ${:.2}", compute_cost);

    let storage_cost = engine.estimate_cost(UsageType::Storage, 150.0).await?;
    println!("150 GB storage would cost: ${:.2}\n", storage_cost);

    // Demo 6: 生成账单
    println!("--- Demo 6: Invoice Generation ---");

    let invoice = engine
        .generate_invoice(user_id, period_start, period_end)
        .await?;

    println!("Invoice #{}", invoice.invoice_number);
    println!("Status: {:?}", invoice.status);
    println!("Period: {} to {}",
        invoice.period_start.format("%Y-%m-%d"),
        invoice.period_end.format("%Y-%m-%d")
    );
    println!("\nItems:");
    for item in &invoice.items {
        println!("  {:?}: {} {} @ ${:.2} = ${:.2}",
            item.usage_type,
            item.quantity,
            item.unit,
            item.unit_price,
            item.subtotal
        );
    }
    println!("\nSubtotal: ${:.2}", invoice.subtotal);
    println!("Tax: ${:.2}", invoice.tax);
    println!("Total: ${:.2}\n", invoice.total);

    // Demo 7: 支付账单
    println!("--- Demo 7: Invoice Payment ---");
    println!("Marking invoice as paid...");

    let paid_invoice = engine.mark_invoice_paid(invoice.id).await?;
    println!("✓ Invoice paid");
    println!("Status: {:?}", paid_invoice.status);
    println!("Paid at: {}\n", paid_invoice.paid_at.unwrap().format("%Y-%m-%d %H:%M:%S"));

    // Demo 8: 月度账单
    println!("--- Demo 8: Monthly Invoice ---");

    // 创建新用户并记录使用量
    let user3_id = Uuid::new_v4();

    tracker
        .record_usage(user3_id, UsageType::ApiCall, 2000.0, "calls".to_string())
        .await?;
    tracker
        .record_usage(user3_id, UsageType::ComputeHours, 20.0, "hours".to_string())
        .await?;
    tracker
        .record_usage(user3_id, UsageType::Storage, 250.0, "GB".to_string())
        .await?;

    let now = Utc::now();
    let monthly_invoice = engine
        .generate_monthly_invoice(user3_id, now.year(), now.month())
        .await?;

    println!("Monthly Invoice #{}", monthly_invoice.invoice_number);
    println!("Total: ${:.2}", monthly_invoice.total);
    println!("Due Date: {}\n", monthly_invoice.due_date.format("%Y-%m-%d"));

    // Demo 9: 配额超限
    println!("--- Demo 9: Quota Exceeded ---");
    let user4_id = Uuid::new_v4();

    // 设置较小的配额
    tracker
        .set_quota(user4_id, UsageType::ApiCall, 50.0, 30)
        .await?;

    println!("Set quota: 50 API calls");

    // 尝试超出配额
    let result = tracker
        .record_usage(user4_id, UsageType::ApiCall, 60.0, "calls".to_string())
        .await;

    if result.is_err() {
        println!("✗ Usage blocked: Quota exceeded\n");
    }

    // Demo 10: 配额重置
    println!("--- Demo 10: Quota Reset ---");

    // 先使用一些配额
    tracker
        .record_usage(user2_id, UsageType::ApiCall, 200.0, "calls".to_string())
        .await?;

    let before_reset = tracker.get_quota(user2_id, UsageType::ApiCall).await.unwrap();
    println!("Before reset - Used: {}", before_reset.used);

    // 重置配额
    tracker.reset_quota(user2_id, UsageType::ApiCall).await?;

    let after_reset = tracker.get_quota(user2_id, UsageType::ApiCall).await.unwrap();
    println!("After reset - Used: {}", after_reset.used);
    println!("✓ Quota reset successful\n");

    // Demo 11: 获取用户所有账单
    println!("--- Demo 11: User Invoices ---");
    let user_invoices = engine.get_user_invoices(user_id).await;
    println!("User has {} invoice(s)", user_invoices.len());

    for (i, inv) in user_invoices.iter().enumerate() {
        println!("  {}. Invoice #{} - ${:.2} ({:?})",
            i + 1,
            inv.invoice_number,
            inv.total,
            inv.status
        );
    }
    println!();

    // Demo 12: 所有计费规则
    println!("--- Demo 12: All Billing Rules ---");
    let all_rules = engine.get_all_rules().await;
    println!("Active billing rules: {}", all_rules.len());

    for rule in &all_rules {
        println!("  - {}: {:?}", rule.name, rule.pricing_model);
    }

    println!("\n=== Demo Complete ===");

    Ok(())
}
