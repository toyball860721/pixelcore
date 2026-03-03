use pixelcore_payment::{
    AccountManager, AccountType, PaymentGateway, GatewayConfig, SettlementManager,
    SettlementType,
};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Payment System Demo ===\n");

    // 创建账户管理器
    let account_manager = AccountManager::new();

    // Demo 1: 创建账户
    println!("--- Demo 1: Create Accounts ---");
    let alice_id = Uuid::new_v4();
    let bob_id = Uuid::new_v4();
    let charlie_id = Uuid::new_v4();

    let alice = account_manager
        .create_account(alice_id, AccountType::Personal)
        .await?;
    let bob = account_manager
        .create_account(bob_id, AccountType::Personal)
        .await?;
    let charlie = account_manager
        .create_account(charlie_id, AccountType::Business)
        .await?;

    println!("Alice's account: {}", alice.id);
    println!("Bob's account: {}", bob.id);
    println!("Charlie's account: {} (Business)", charlie.id);
    println!();

    // Demo 2: 充值
    println!("--- Demo 2: Deposit ---");
    let deposit_tx = account_manager
        .deposit(alice.id, 1000.0, "Initial deposit".to_string())
        .await?;

    println!("Alice deposited {} PixelCoin", deposit_tx.amount);
    println!("Transaction ID: {}", deposit_tx.id);

    let balance = account_manager.get_balance(alice.id).await?;
    println!("Alice's balance: {} PixelCoin\n", balance);

    // Demo 3: 转账
    println!("--- Demo 3: Transfer ---");
    let transfer_tx = account_manager
        .transfer(alice.id, bob.id, 300.0, "Payment for service".to_string())
        .await?;

    println!("Alice transferred {} PixelCoin to Bob", transfer_tx.amount);

    let alice_balance = account_manager.get_balance(alice.id).await?;
    let bob_balance = account_manager.get_balance(bob.id).await?;

    println!("Alice's balance: {} PixelCoin", alice_balance);
    println!("Bob's balance: {} PixelCoin\n", bob_balance);

    // Demo 4: 支付网关 (带手续费)
    println!("--- Demo 4: Payment Gateway with Fees ---");
    let gateway_config = GatewayConfig {
        deposit_fee_rate: 0.0,
        withdrawal_fee_rate: 0.02,  // 2% 提现手续费
        transfer_fee_rate: 0.01,    // 1% 转账手续费
        min_deposit: 1.0,
        min_withdrawal: 10.0,
        max_transaction: 100000.0,
    };

    let gateway = PaymentGateway::new(account_manager.clone(), gateway_config);

    // 充值 (免手续费)
    let deposit_tx = gateway.deposit(charlie.id, 500.0).await?;
    println!("Charlie deposited {} PixelCoin (fee: {})",
        deposit_tx.amount, deposit_tx.fee);

    // 转账 (1% 手续费)
    let transfer_tx = gateway.transfer(charlie.id, alice.id, 200.0).await?;
    println!("Charlie transferred {} PixelCoin to Alice (fee: {})",
        transfer_tx.amount, transfer_tx.fee);

    let charlie_balance = account_manager.get_balance(charlie.id).await?;
    println!("Charlie's balance: {} PixelCoin\n", charlie_balance);

    // Demo 5: 提现 (带手续费)
    println!("--- Demo 5: Withdrawal with Fee ---");
    let withdraw_tx = gateway.withdraw(bob.id, 100.0).await?;
    println!("Bob withdrew {} PixelCoin (fee: {})",
        withdraw_tx.amount, withdraw_tx.fee);

    let bob_balance = account_manager.get_balance(bob.id).await?;
    println!("Bob's balance: {} PixelCoin\n", bob_balance);

    // Demo 6: 即时结算
    println!("--- Demo 6: Immediate Settlement ---");
    let settlement_manager = SettlementManager::new(account_manager.clone());

    let seller = account_manager
        .create_account(Uuid::new_v4(), AccountType::Business)
        .await?;
    let buyer = account_manager
        .create_account(Uuid::new_v4(), AccountType::Personal)
        .await?;

    // 买方充值
    account_manager
        .deposit(buyer.id, 500.0, "Buyer deposit".to_string())
        .await?;

    // 创建即时结算
    let transaction_id = Uuid::new_v4();
    let settlement = settlement_manager
        .create_immediate_settlement(transaction_id, seller.id, buyer.id, 250.0)
        .await?;

    println!("Settlement ID: {}", settlement.id);
    println!("Settlement Type: {:?}", settlement.settlement_type);
    println!("Settlement Status: {:?}", settlement.status);
    println!("Amount: {} PixelCoin", settlement.amount);

    let seller_balance = account_manager.get_balance(seller.id).await?;
    let buyer_balance = account_manager.get_balance(buyer.id).await?;

    println!("Seller's balance: {} PixelCoin", seller_balance);
    println!("Buyer's balance: {} PixelCoin\n", buyer_balance);

    // Demo 7: 托管结算
    println!("--- Demo 7: Escrow Settlement ---");
    let escrow_account = account_manager
        .create_account(Uuid::new_v4(), AccountType::Escrow)
        .await?;

    let seller2 = account_manager
        .create_account(Uuid::new_v4(), AccountType::Business)
        .await?;
    let buyer2 = account_manager
        .create_account(Uuid::new_v4(), AccountType::Personal)
        .await?;

    // 买方充值
    account_manager
        .deposit(buyer2.id, 1000.0, "Buyer deposit".to_string())
        .await?;

    // 创建托管结算
    let transaction_id2 = Uuid::new_v4();
    let escrow_settlement = settlement_manager
        .create_escrow_settlement(
            transaction_id2,
            seller2.id,
            buyer2.id,
            escrow_account.id,
            400.0,
        )
        .await?;

    println!("Escrow Settlement ID: {}", escrow_settlement.id);
    println!("Status: {:?}", escrow_settlement.status);

    let escrow_balance = account_manager.get_balance(escrow_account.id).await?;
    println!("Escrow account balance: {} PixelCoin", escrow_balance);

    // 释放托管资金
    println!("\nReleasing escrow funds to seller...");
    settlement_manager
        .release_escrow(escrow_settlement.id, escrow_account.id)
        .await?;

    let seller2_balance = account_manager.get_balance(seller2.id).await?;
    println!("Seller's balance after release: {} PixelCoin\n", seller2_balance);

    // Demo 8: 分账
    println!("--- Demo 8: Split Payment ---");
    let platform = account_manager
        .create_account(Uuid::new_v4(), AccountType::System)
        .await?;
    let merchant = account_manager
        .create_account(Uuid::new_v4(), AccountType::Business)
        .await?;
    let payer = account_manager
        .create_account(Uuid::new_v4(), AccountType::Personal)
        .await?;

    // 付款方充值
    account_manager
        .deposit(payer.id, 1000.0, "Payer deposit".to_string())
        .await?;

    // 分账: 90% 给商家, 10% 给平台
    let splits = vec![(merchant.id, 0.9), (platform.id, 0.1)];

    let split_settlements = settlement_manager
        .split_payment(Uuid::new_v4(), payer.id, splits, 500.0)
        .await?;

    println!("Split payment completed: {} settlements", split_settlements.len());

    let merchant_balance = account_manager.get_balance(merchant.id).await?;
    let platform_balance = account_manager.get_balance(platform.id).await?;

    println!("Merchant received: {} PixelCoin (90%)", merchant_balance);
    println!("Platform received: {} PixelCoin (10%)\n", platform_balance);

    // Demo 9: 交易历史
    println!("--- Demo 9: Transaction History ---");
    let history = account_manager.get_transaction_history(alice.id).await?;

    println!("Alice's transaction history ({} transactions):", history.len());
    for (i, tx) in history.iter().enumerate() {
        println!("  {}. {:?} - {} PixelCoin (Status: {:?})",
            i + 1, tx.payment_type, tx.amount, tx.status);
    }
    println!();

    // Demo 10: 账户冻结
    println!("--- Demo 10: Account Freeze ---");
    println!("Freezing Bob's account...");
    account_manager.freeze_account(bob.id).await?;

    let result = account_manager
        .withdraw(bob.id, 50.0, "Attempt withdrawal".to_string())
        .await;

    if result.is_err() {
        println!("✓ Withdrawal blocked (account frozen)");
    }

    println!("Unfreezing Bob's account...");
    account_manager.unfreeze_account(bob.id).await?;

    let result = account_manager
        .withdraw(bob.id, 50.0, "Successful withdrawal".to_string())
        .await;

    if result.is_ok() {
        println!("✓ Withdrawal successful (account active)\n");
    }

    // 最终余额汇总
    println!("--- Final Balances ---");
    let accounts = account_manager.list_accounts().await;
    for account in accounts.iter().take(5) {
        let balance = account_manager.get_balance(account.id).await?;
        println!("{:?} account: {} PixelCoin", account.account_type, balance);
    }

    println!("\n=== Demo Complete ===");

    Ok(())
}
