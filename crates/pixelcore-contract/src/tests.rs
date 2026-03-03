use super::*;
use chrono::Duration;
use pixelcore_transaction::{Transaction, TransactionType};
use uuid::Uuid;

#[test]
fn test_contract_creation() {
    let party_a = Uuid::new_v4();
    let party_b = Uuid::new_v4();

    let contract = SmartContract::new(ContractType::Service, party_a, party_b, 100.0);

    assert_eq!(contract.party_a, party_a);
    assert_eq!(contract.party_b, party_b);
    assert_eq!(contract.amount, 100.0);
    assert_eq!(contract.status, ContractStatus::Draft);
    assert!(contract.signed_at.is_none());
}

#[test]
fn test_contract_signing() {
    let party_a = Uuid::new_v4();
    let party_b = Uuid::new_v4();

    let mut contract = SmartContract::new(ContractType::Service, party_a, party_b, 100.0);

    // Sign contract
    contract.sign();
    assert_eq!(contract.status, ContractStatus::Active);
    assert!(contract.signed_at.is_some());
    assert!(contract.start_time.is_some());
}

#[test]
fn test_contract_execution_lifecycle() {
    let party_a = Uuid::new_v4();
    let party_b = Uuid::new_v4();

    let mut contract = SmartContract::new(ContractType::Service, party_a, party_b, 100.0);

    // Sign and activate
    contract.sign();
    assert_eq!(contract.status, ContractStatus::Active);

    // Start execution
    contract.start_execution();
    assert_eq!(contract.status, ContractStatus::Executing);

    // Complete execution
    contract.complete();
    assert_eq!(contract.status, ContractStatus::Completed);
    assert!(contract.completed_at.is_some());
}

#[test]
fn test_contract_termination() {
    let party_a = Uuid::new_v4();
    let party_b = Uuid::new_v4();

    let mut contract = SmartContract::new(ContractType::Service, party_a, party_b, 100.0);

    contract.sign();

    // Terminate contract
    contract.terminate();
    assert_eq!(contract.status, ContractStatus::Terminated);
    assert!(contract.completed_at.is_some());
}

#[test]
fn test_contract_dispute() {
    let party_a = Uuid::new_v4();
    let party_b = Uuid::new_v4();

    let mut contract = SmartContract::new(ContractType::Service, party_a, party_b, 100.0);

    contract.sign();
    contract.start_execution();

    // Raise dispute
    contract.dispute();
    assert_eq!(contract.status, ContractStatus::Disputed);
}

#[test]
fn test_contract_term() {
    let mut term = ContractTerm::new(
        "Test Term".to_string(),
        "This is a test term".to_string(),
        true,
    );

    // Add precondition
    term.add_precondition(Condition::AmountCondition {
        min: Some(100.0),
        max: Some(1000.0),
    });

    // Add postcondition
    term.add_postcondition(Condition::StatusCondition {
        required_status: "completed".to_string(),
    });

    assert_eq!(term.preconditions.len(), 1);
    assert_eq!(term.postconditions.len(), 1);
}

#[test]
fn test_condition_checking() {
    // Test amount condition
    let amount_condition = Condition::AmountCondition {
        min: Some(100.0),
        max: Some(1000.0),
    };

    let context_valid = serde_json::json!({"amount": 500.0});
    assert!(amount_condition.check(&context_valid));

    let context_invalid = serde_json::json!({"amount": 50.0});
    assert!(!amount_condition.check(&context_invalid));

    // Test status condition
    let status_condition = Condition::StatusCondition {
        required_status: "executing".to_string(),
    };

    let context_valid = serde_json::json!({"status": "executing"});
    assert!(status_condition.check(&context_valid));

    let context_invalid = serde_json::json!({"status": "pending"});
    assert!(!status_condition.check(&context_invalid));
}

#[test]
fn test_validator_basic_fields() {
    let validator = ContractValidator::new();

    let party_a = Uuid::new_v4();
    let party_b = Uuid::new_v4();

    // Valid contract
    let mut contract = SmartContract::new(ContractType::Service, party_a, party_b, 100.0);
    contract.add_term(ContractTerm::new(
        "Test Term".to_string(),
        "Test description".to_string(),
        true,
    ));

    let result = validator.validate_contract(&contract);
    assert!(result.is_valid);

    // Invalid contract (negative amount)
    let invalid_contract = SmartContract::new(ContractType::Service, party_a, party_b, -100.0);
    let result = validator.validate_contract(&invalid_contract);
    assert!(!result.is_valid);
    assert!(result.errors.iter().any(|e| e.contains("cannot be negative")));
}

#[test]
fn test_validator_parties() {
    let validator = ContractValidator::new();

    let same_id = Uuid::new_v4();

    // Same party A and party B
    let contract = SmartContract::new(ContractType::Service, same_id, same_id, 100.0);

    let result = validator.validate_contract(&contract);
    assert!(!result.is_valid);
    assert!(result.errors.iter().any(|e| e.contains("cannot be the same")));
}

#[test]
fn test_validator_status() {
    let validator = ContractValidator::new();

    let party_a = Uuid::new_v4();
    let party_b = Uuid::new_v4();

    let mut contract = SmartContract::new(ContractType::Service, party_a, party_b, 100.0);
    contract.add_term(ContractTerm::new(
        "Test Term".to_string(),
        "Test description".to_string(),
        true,
    ));

    // Active contract without signed_at
    contract.status = ContractStatus::Active;
    let result = validator.validate_contract(&contract);
    assert!(!result.is_valid);
    assert!(result.errors.iter().any(|e| e.contains("signed_at")));
}

#[tokio::test]
async fn test_executor_register_contract() {
    let executor = ContractExecutor::new();

    let party_a = Uuid::new_v4();
    let party_b = Uuid::new_v4();

    let mut contract = SmartContract::new(ContractType::Service, party_a, party_b, 100.0);

    // Cannot register draft contract
    assert!(executor.register_contract(contract.clone()).await.is_err());

    // Sign contract
    contract.sign();

    // Now can register
    assert!(executor.register_contract(contract).await.is_ok());
}

#[tokio::test]
async fn test_executor_execute_contract() {
    let executor = ContractExecutor::new();

    let party_a = Uuid::new_v4();
    let party_b = Uuid::new_v4();

    let mut contract = SmartContract::new(ContractType::Service, party_a, party_b, 100.0);

    // Add a simple term
    let mut term = ContractTerm::new(
        "Service Delivery".to_string(),
        "Provide service".to_string(),
        true,
    );
    term.add_precondition(Condition::StatusCondition {
        required_status: "executing".to_string(),
    });
    contract.add_term(term);

    contract.sign();
    executor.register_contract(contract.clone()).await.unwrap();

    // Create a transaction with ServiceCall type
    let transaction = Transaction::new(
        party_a,
        party_b,
        TransactionType::ServiceCall {
            agent_id: party_b,
            skill_name: "test_skill".to_string(),
            input: serde_json::json!({}),
        },
        100.0,
    );

    // Execute contract
    let result = executor.execute_contract(contract.id, &transaction).await;
    assert!(result.is_ok());
}

#[test]
fn test_service_contract_template() {
    let party_a = Uuid::new_v4();
    let party_b = Uuid::new_v4();

    let contract = ContractTemplate::service_contract(
        party_a,
        party_b,
        "AI Model Training".to_string(),
        100.0,
        Duration::days(7),
    );

    assert_eq!(contract.party_a, party_a);
    assert_eq!(contract.party_b, party_b);
    assert_eq!(contract.amount, 100.0);
    assert!(!contract.terms.is_empty());
    assert!(matches!(contract.contract_type, ContractType::Service));
}

#[test]
fn test_data_purchase_contract_template() {
    let party_a = Uuid::new_v4();
    let party_b = Uuid::new_v4();

    let contract = ContractTemplate::data_purchase_contract(
        party_a,
        party_b,
        "Training Dataset".to_string(),
        500.0,
    );

    assert_eq!(contract.amount, 500.0);
    assert!(!contract.terms.is_empty());
    assert!(matches!(contract.contract_type, ContractType::Data));
    // Should have payment and delivery terms
    assert!(contract.terms.len() >= 2);
}

#[test]
fn test_subscription_contract_template() {
    let party_a = Uuid::new_v4();
    let party_b = Uuid::new_v4();

    let contract = ContractTemplate::subscription_contract(
        party_a,
        party_b,
        "API Access".to_string(),
        50.0,
        Duration::days(30),
    );

    assert_eq!(contract.amount, 50.0);
    assert!(!contract.terms.is_empty());
    assert!(matches!(contract.contract_type, ContractType::Subscription));
}

#[test]
fn test_compute_contract_template() {
    let party_a = Uuid::new_v4();
    let party_b = Uuid::new_v4();

    let contract = ContractTemplate::compute_contract(
        party_a,
        party_b,
        "GPU Compute".to_string(),
        200.0,
        100,
    );

    assert_eq!(contract.amount, 200.0);
    assert!(!contract.terms.is_empty());
    assert!(matches!(contract.contract_type, ContractType::Compute));
}
