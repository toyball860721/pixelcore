use super::*;
use uuid::Uuid;

#[tokio::test]
async fn test_account_creation() {
    let manager = AccountManager::new();
    let owner_id = Uuid::new_v4();

    let account = manager
        .create_account(owner_id, AccountType::Personal)
        .await
        .unwrap();

    assert_eq!(account.owner_id, owner_id);
    assert_eq!(account.account_type, AccountType::Personal);
    assert_eq!(account.status, AccountStatus::Active);
    assert_eq!(account.balance, 0.0);
}

#[tokio::test]
async fn test_deposit() {
    let manager = AccountManager::new();
    let owner_id = Uuid::new_v4();

    let account = manager
        .create_account(owner_id, AccountType::Personal)
        .await
        .unwrap();

    let transaction = manager
        .deposit(account.id, 100.0, "Test deposit".to_string())
        .await
        .unwrap();

    assert_eq!(transaction.payment_type, PaymentType::Deposit);
    assert_eq!(transaction.amount, 100.0);
    assert_eq!(transaction.status, PaymentStatus::Success);

    let balance = manager.get_balance(account.id).await.unwrap();
    assert_eq!(balance, 100.0);
}

#[tokio::test]
async fn test_withdraw() {
    let manager = AccountManager::new();
    let owner_id = Uuid::new_v4();

    let account = manager
        .create_account(owner_id, AccountType::Personal)
        .await
        .unwrap();

    // 先充值
    manager
        .deposit(account.id, 100.0, "Test deposit".to_string())
        .await
        .unwrap();

    // 提现
    let transaction = manager
        .withdraw(account.id, 50.0, "Test withdrawal".to_string())
        .await
        .unwrap();

    assert_eq!(transaction.payment_type, PaymentType::Withdrawal);
    assert_eq!(transaction.amount, 50.0);

    let balance = manager.get_balance(account.id).await.unwrap();
    assert_eq!(balance, 50.0);
}

#[tokio::test]
async fn test_transfer() {
    let manager = AccountManager::new();

    let account1 = manager
        .create_account(Uuid::new_v4(), AccountType::Personal)
        .await
        .unwrap();

    let account2 = manager
        .create_account(Uuid::new_v4(), AccountType::Personal)
        .await
        .unwrap();

    // 给账户1充值
    manager
        .deposit(account1.id, 100.0, "Test deposit".to_string())
        .await
        .unwrap();

    // 转账
    let transaction = manager
        .transfer(account1.id, account2.id, 30.0, "Test transfer".to_string())
        .await
        .unwrap();

    assert_eq!(transaction.payment_type, PaymentType::Transfer);
    assert_eq!(transaction.amount, 30.0);

    let balance1 = manager.get_balance(account1.id).await.unwrap();
    let balance2 = manager.get_balance(account2.id).await.unwrap();

    assert_eq!(balance1, 70.0);
    assert_eq!(balance2, 30.0);
}

#[tokio::test]
async fn test_insufficient_balance() {
    let manager = AccountManager::new();
    let owner_id = Uuid::new_v4();

    let account = manager
        .create_account(owner_id, AccountType::Personal)
        .await
        .unwrap();

    // 尝试提现但余额不足
    let result = manager
        .withdraw(account.id, 50.0, "Test withdrawal".to_string())
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_freeze_unfreeze() {
    let manager = AccountManager::new();
    let owner_id = Uuid::new_v4();

    let account = manager
        .create_account(owner_id, AccountType::Personal)
        .await
        .unwrap();

    // 充值
    manager
        .deposit(account.id, 100.0, "Test deposit".to_string())
        .await
        .unwrap();

    // 冻结账户
    manager.freeze_account(account.id).await.unwrap();

    // 尝试提现应该失败
    let result = manager
        .withdraw(account.id, 50.0, "Test withdrawal".to_string())
        .await;
    assert!(result.is_err());

    // 解冻账户
    manager.unfreeze_account(account.id).await.unwrap();

    // 现在应该可以提现
    let result = manager
        .withdraw(account.id, 50.0, "Test withdrawal".to_string())
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_payment_gateway_deposit() {
    let manager = AccountManager::new();
    let gateway = PaymentGateway::with_defaults(manager.clone());

    let account = manager
        .create_account(Uuid::new_v4(), AccountType::Personal)
        .await
        .unwrap();

    let transaction = gateway.deposit(account.id, 100.0).await.unwrap();

    assert_eq!(transaction.payment_type, PaymentType::Deposit);
    assert_eq!(transaction.fee, 0.0); // 充值免手续费

    let balance = manager.get_balance(account.id).await.unwrap();
    assert_eq!(balance, 100.0);
}

#[tokio::test]
async fn test_payment_gateway_withdrawal_with_fee() {
    let manager = AccountManager::new();
    let gateway = PaymentGateway::with_defaults(manager.clone());

    let account = manager
        .create_account(Uuid::new_v4(), AccountType::Personal)
        .await
        .unwrap();

    // 充值
    manager
        .deposit(account.id, 100.0, "Test deposit".to_string())
        .await
        .unwrap();

    // 提现 (会扣除手续费)
    let transaction = gateway.withdraw(account.id, 50.0).await.unwrap();

    assert_eq!(transaction.payment_type, PaymentType::Withdrawal);
    assert_eq!(transaction.fee, 0.5); // 1% 手续费

    let balance = manager.get_balance(account.id).await.unwrap();
    assert_eq!(balance, 49.5); // 100 - 50 - 0.5
}

#[tokio::test]
async fn test_payment_gateway_transfer_with_fee() {
    let manager = AccountManager::new();
    let gateway = PaymentGateway::with_defaults(manager.clone());

    let account1 = manager
        .create_account(Uuid::new_v4(), AccountType::Personal)
        .await
        .unwrap();

    let account2 = manager
        .create_account(Uuid::new_v4(), AccountType::Personal)
        .await
        .unwrap();

    // 充值
    manager
        .deposit(account1.id, 100.0, "Test deposit".to_string())
        .await
        .unwrap();

    // 转账 (会扣除手续费)
    let transaction = gateway
        .transfer(account1.id, account2.id, 50.0)
        .await
        .unwrap();

    assert_eq!(transaction.fee, 0.25); // 0.5% 手续费

    let balance1 = manager.get_balance(account1.id).await.unwrap();
    let balance2 = manager.get_balance(account2.id).await.unwrap();

    assert_eq!(balance1, 49.75); // 100 - 50 - 0.25
    assert_eq!(balance2, 50.0);
}

#[tokio::test]
async fn test_immediate_settlement() {
    let manager = AccountManager::new();
    let settlement_manager = SettlementManager::new(manager.clone());

    let buyer = manager
        .create_account(Uuid::new_v4(), AccountType::Personal)
        .await
        .unwrap();

    let seller = manager
        .create_account(Uuid::new_v4(), AccountType::Personal)
        .await
        .unwrap();

    // 买方充值
    manager
        .deposit(buyer.id, 100.0, "Test deposit".to_string())
        .await
        .unwrap();

    // 创建即时结算
    let transaction_id = Uuid::new_v4();
    let settlement = settlement_manager
        .create_immediate_settlement(transaction_id, seller.id, buyer.id, 50.0)
        .await
        .unwrap();

    assert_eq!(settlement.settlement_type, SettlementType::Immediate);
    assert_eq!(settlement.status, SettlementStatus::Settled);

    let buyer_balance = manager.get_balance(buyer.id).await.unwrap();
    let seller_balance = manager.get_balance(seller.id).await.unwrap();

    assert_eq!(buyer_balance, 50.0);
    assert_eq!(seller_balance, 50.0);
}

#[tokio::test]
async fn test_escrow_settlement() {
    let manager = AccountManager::new();
    let settlement_manager = SettlementManager::new(manager.clone());

    let buyer = manager
        .create_account(Uuid::new_v4(), AccountType::Personal)
        .await
        .unwrap();

    let seller = manager
        .create_account(Uuid::new_v4(), AccountType::Personal)
        .await
        .unwrap();

    let escrow = manager
        .create_account(Uuid::new_v4(), AccountType::Escrow)
        .await
        .unwrap();

    // 买方充值
    manager
        .deposit(buyer.id, 100.0, "Test deposit".to_string())
        .await
        .unwrap();

    // 创建托管结算
    let transaction_id = Uuid::new_v4();
    let settlement = settlement_manager
        .create_escrow_settlement(transaction_id, seller.id, buyer.id, escrow.id, 50.0)
        .await
        .unwrap();

    assert_eq!(settlement.settlement_type, SettlementType::Escrow);
    assert_eq!(settlement.status, SettlementStatus::Pending);

    // 资金应该在托管账户
    let escrow_balance = manager.get_balance(escrow.id).await.unwrap();
    assert_eq!(escrow_balance, 50.0);

    // 释放托管资金给卖方
    settlement_manager
        .release_escrow(settlement.id, escrow.id)
        .await
        .unwrap();

    let seller_balance = manager.get_balance(seller.id).await.unwrap();
    assert_eq!(seller_balance, 50.0);
}

#[tokio::test]
async fn test_split_payment() {
    let manager = AccountManager::new();
    let settlement_manager = SettlementManager::new(manager.clone());

    let payer = manager
        .create_account(Uuid::new_v4(), AccountType::Personal)
        .await
        .unwrap();

    let recipient1 = manager
        .create_account(Uuid::new_v4(), AccountType::Personal)
        .await
        .unwrap();

    let recipient2 = manager
        .create_account(Uuid::new_v4(), AccountType::Personal)
        .await
        .unwrap();

    // 充值
    manager
        .deposit(payer.id, 100.0, "Test deposit".to_string())
        .await
        .unwrap();

    // 分账: 60% 给 recipient1, 40% 给 recipient2
    let splits = vec![(recipient1.id, 0.6), (recipient2.id, 0.4)];

    let settlements = settlement_manager
        .split_payment(Uuid::new_v4(), payer.id, splits, 100.0)
        .await
        .unwrap();

    assert_eq!(settlements.len(), 2);

    let balance1 = manager.get_balance(recipient1.id).await.unwrap();
    let balance2 = manager.get_balance(recipient2.id).await.unwrap();

    assert_eq!(balance1, 60.0);
    assert_eq!(balance2, 40.0);
}
