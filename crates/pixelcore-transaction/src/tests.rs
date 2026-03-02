use super::*;

#[test]
fn test_transaction_creation() {
    let buyer_id = uuid::Uuid::new_v4();
    let seller_id = uuid::Uuid::new_v4();
    let agent_id = uuid::Uuid::new_v4();

    let tx_type = TransactionType::ServiceCall {
        agent_id,
        skill_name: "calculate".to_string(),
        input: serde_json::json!({"expression": "1+1"}),
    };

    let transaction = Transaction::new(buyer_id, seller_id, tx_type, 0.01);

    assert_eq!(transaction.status, TransactionStatus::Pending);
    assert_eq!(transaction.amount, 0.01);
    assert_eq!(transaction.currency, "PixelCoin");
}

#[test]
fn test_state_machine() {
    use TransactionStatus::*;

    assert!(TransactionStateMachine::can_transition(Pending, Confirmed));
    assert!(TransactionStateMachine::can_transition(Confirmed, Executing));
    assert!(TransactionStateMachine::can_transition(Executing, Completed));

    assert!(!TransactionStateMachine::can_transition(Completed, Executing));
    assert!(!TransactionStateMachine::can_transition(Pending, Completed));
}
