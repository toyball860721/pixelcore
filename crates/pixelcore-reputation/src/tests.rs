use super::*;
use uuid::Uuid;
use anyhow::Result;

#[test]
fn test_review_creation() {
    let transaction_id = Uuid::new_v4();
    let agent_id = Uuid::new_v4();
    let reviewer_id = Uuid::new_v4();

    let review = Review::new(
        transaction_id,
        agent_id,
        reviewer_id,
        5,
        "Excellent service!".to_string(),
    );

    assert_eq!(review.rating, 5);
    assert_eq!(review.comment, "Excellent service!");
    assert!(!review.verified);

    // 测试评分限制
    let review2 = Review::new(
        transaction_id,
        agent_id,
        reviewer_id,
        10, // 超出范围
        "Test".to_string(),
    );
    assert_eq!(review2.rating, 5); // 应该被限制为 5
}

#[test]
fn test_reputation_level() {
    assert_eq!(ReputationLevel::from_transaction_count(5), ReputationLevel::Newcomer);
    assert_eq!(ReputationLevel::from_transaction_count(30), ReputationLevel::Regular);
    assert_eq!(ReputationLevel::from_transaction_count(100), ReputationLevel::Excellent);
    assert_eq!(ReputationLevel::from_transaction_count(500), ReputationLevel::Expert);

    assert_eq!(ReputationLevel::Newcomer.name(), "新手");
    assert_eq!(ReputationLevel::Expert.name(), "专家");
}

#[test]
fn test_reputation_record() {
    let agent_id = Uuid::new_v4();
    let mut record = ReputationRecord::new(agent_id);

    assert_eq!(record.score, 0.0);
    assert_eq!(record.total_transactions, 0);
    assert_eq!(record.level, ReputationLevel::Newcomer);

    // 添加评价
    let review = Review::new(
        Uuid::new_v4(),
        agent_id,
        Uuid::new_v4(),
        5,
        "Great!".to_string(),
    );
    record.add_review(review);

    assert_eq!(record.reviews.len(), 1);
    assert_eq!(record.average_rating(), 5.0);
}

#[test]
fn test_reputation_calculator() {
    let calculator = ReputationCalculator::new();
    let agent_id = Uuid::new_v4();
    let mut record = ReputationRecord::new(agent_id);

    // 添加一些评价
    for rating in [5, 5, 4, 5, 4] {
        let review = Review::new(
            Uuid::new_v4(),
            agent_id,
            Uuid::new_v4(),
            rating,
            "Test".to_string(),
        );
        record.add_review(review);
    }

    // 记录交易
    calculator.record_transaction(&mut record, true, 500);
    calculator.record_transaction(&mut record, true, 600);
    calculator.record_transaction(&mut record, false, 1000);

    assert_eq!(record.total_transactions, 3);
    assert_eq!(record.successful_transactions, 2);
    assert!(record.score > 0.0);
    assert!(record.score <= 5.0);

    // 测试成功率
    let success_rate = record.success_rate();
    assert!((success_rate - 0.666).abs() < 0.01);
}

#[test]
fn test_reputation_manager() -> Result<()> {
    let manager = ReputationManager::in_memory()?;
    let agent_id = Uuid::new_v4();

    // 添加评价
    let review_id = manager.add_review(
        Uuid::new_v4(),
        agent_id,
        Uuid::new_v4(),
        5,
        "Excellent service!".to_string(),
    )?;

    // 获取记录
    let record = manager.get_record(&agent_id)?;
    assert!(record.is_some());
    let record = record.unwrap();
    assert_eq!(record.reviews.len(), 1);
    assert_eq!(record.average_rating(), 5.0);

    // 验证评价
    manager.verify_review(&review_id, &agent_id)?;

    // 记录交易
    manager.record_transaction(&agent_id, true, 500)?;
    manager.record_transaction(&agent_id, true, 600)?;

    let updated_record = manager.get_record(&agent_id)?.unwrap();
    assert_eq!(updated_record.total_transactions, 2);
    assert_eq!(updated_record.successful_transactions, 2);

    // 获取统计
    let stats = manager.get_stats(&agent_id)?;
    assert!(stats.is_some());
    let stats = stats.unwrap();
    assert_eq!(stats.five_star_count, 1);
    assert_eq!(stats.total_reviews, 1);

    Ok(())
}

#[test]
fn test_anomaly_detection() -> Result<()> {
    let manager = ReputationManager::in_memory()?;
    let agent_id = Uuid::new_v4();

    // 添加大量 5 星评价 (可能的刷单)
    for _ in 0..15 {
        manager.add_review(
            Uuid::new_v4(),
            agent_id,
            Uuid::new_v4(),
            5,
            "Perfect!".to_string(),
        )?;
    }

    // 记录交易 (100% 成功率)
    for _ in 0..15 {
        manager.record_transaction(&agent_id, true, 500)?;
    }

    let anomalies = manager.detect_anomaly(&agent_id)?;
    assert!(!anomalies.is_empty());
    // 应该检测到评分分布异常和成功率异常

    Ok(())
}

#[test]
fn test_reputation_stats() {
    let agent_id = Uuid::new_v4();
    let mut record = ReputationRecord::new(agent_id);

    // 添加不同评分的评价
    for rating in [5, 5, 4, 4, 3, 2, 1] {
        let review = Review::new(
            Uuid::new_v4(),
            agent_id,
            Uuid::new_v4(),
            rating,
            "Test".to_string(),
        );
        record.add_review(review);
    }

    let stats = ReputationStats::from_record(&record);
    assert_eq!(stats.five_star_count, 2);
    assert_eq!(stats.four_star_count, 2);
    assert_eq!(stats.three_star_count, 1);
    assert_eq!(stats.two_star_count, 1);
    assert_eq!(stats.one_star_count, 1);
    assert_eq!(stats.total_reviews, 7);
}
