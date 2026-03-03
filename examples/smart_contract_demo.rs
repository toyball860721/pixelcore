use pixelcore_contract::{
    SmartContract, ContractType, ContractTerm, Condition, ContractTemplate,
    ContractExecutor, ContractValidator,
};
use pixelcore_transaction::{Transaction, TransactionType};
use uuid::Uuid;
use chrono::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Smart Contract Engine Demo ===\n");

    // Create parties
    let provider_id = Uuid::new_v4();
    let consumer_id = Uuid::new_v4();

    println!("Provider ID: {}", provider_id);
    println!("Consumer ID: {}\n", consumer_id);

    // Demo 1: Create a service contract using template
    println!("--- Demo 1: Service Contract Template ---");
    let mut service_contract = ContractTemplate::service_contract(
        consumer_id,
        provider_id,
        "AI Model Training".to_string(),
        500.0,
        Duration::days(7),
    );

    println!("Contract ID: {}", service_contract.id);
    println!("Contract Type: {:?}", service_contract.contract_type);
    println!("Amount: {} {}", service_contract.amount, service_contract.currency);
    println!("Terms: {}", service_contract.terms.len());
    for term in &service_contract.terms {
        println!("  - {}: {}", term.name, term.description);
    }
    println!();

    // Demo 2: Validate contract
    println!("--- Demo 2: Contract Validation ---");
    let validator = ContractValidator::new();
    let validation_result = validator.validate_contract(&service_contract);

    println!("Is Valid: {}", validation_result.is_valid);
    if !validation_result.errors.is_empty() {
        println!("Errors:");
        for error in &validation_result.errors {
            println!("  - {}", error);
        }
    }
    if !validation_result.warnings.is_empty() {
        println!("Warnings:");
        for warning in &validation_result.warnings {
            println!("  - {}", warning);
        }
    }
    println!();

    // Demo 3: Sign and activate contract
    println!("--- Demo 3: Sign and Activate Contract ---");
    println!("Status before signing: {:?}", service_contract.status);
    service_contract.sign();
    println!("Status after signing: {:?}", service_contract.status);
    println!("Signed at: {:?}", service_contract.signed_at);
    println!();

    // Demo 4: Execute contract
    println!("--- Demo 4: Contract Execution ---");
    let executor = ContractExecutor::new();

    // Register contract
    executor.register_contract(service_contract.clone()).await?;
    println!("Contract registered for execution");

    // Create a transaction
    let mut transaction = Transaction::new(
        consumer_id,
        provider_id,
        TransactionType::ServiceCall {
            agent_id: provider_id,
            skill_name: "ai_training".to_string(),
            input: serde_json::json!({
                "model": "GPT-4",
                "dataset": "custom_data",
            }),
        },
        500.0,
    );

    // Start transaction execution
    transaction.start_execution();
    println!("Transaction status: {:?}", transaction.status);

    // Execute contract
    let execution_result = executor.execute_contract(service_contract.id, &transaction).await?;
    println!("Execution success: {}", execution_result.success);
    if let Some(error) = execution_result.error {
        println!("Execution error: {}", error);
    }
    println!();

    // Demo 5: Data purchase contract
    println!("--- Demo 5: Data Purchase Contract ---");
    let data_contract = ContractTemplate::data_purchase_contract(
        consumer_id,
        provider_id,
        "Training Dataset".to_string(),
        1000.0,
    );

    println!("Contract ID: {}", data_contract.id);
    println!("Contract Type: {:?}", data_contract.contract_type);
    println!("Amount: {} {}", data_contract.amount, data_contract.currency);
    println!("Terms: {}", data_contract.terms.len());
    println!();

    // Demo 6: Subscription contract
    println!("--- Demo 6: Subscription Contract ---");
    let subscription_contract = ContractTemplate::subscription_contract(
        consumer_id,
        provider_id,
        "API Access".to_string(),
        50.0,
        Duration::days(30),
    );

    println!("Contract ID: {}", subscription_contract.id);
    println!("Contract Type: {:?}", subscription_contract.contract_type);
    println!("Monthly Amount: {} {}", subscription_contract.amount, subscription_contract.currency);
    println!("End Time: {:?}", subscription_contract.end_time);
    println!();

    // Demo 7: Compute contract
    println!("--- Demo 7: Compute Resource Contract ---");
    let compute_contract = ContractTemplate::compute_contract(
        consumer_id,
        provider_id,
        "GPU Compute".to_string(),
        200.0,
        100,
    );

    println!("Contract ID: {}", compute_contract.id);
    println!("Contract Type: {:?}", compute_contract.contract_type);
    println!("Amount: {} {}", compute_contract.amount, compute_contract.currency);
    println!();

    // Demo 8: Custom contract with conditions
    println!("--- Demo 8: Custom Contract with Conditions ---");
    let mut custom_contract = SmartContract::new(
        ContractType::Service,
        consumer_id,
        provider_id,
        300.0,
    );

    // Add custom term with time condition
    let mut time_term = ContractTerm::new(
        "Time-Limited Service".to_string(),
        "Service must be completed within 24 hours".to_string(),
        true,
    );
    time_term.add_precondition(Condition::TimeCondition {
        before: Some(chrono::Utc::now() + Duration::hours(24)),
        after: None,
    });
    custom_contract.add_term(time_term);

    // Add term with amount condition
    let mut amount_term = ContractTerm::new(
        "Payment Range".to_string(),
        "Payment must be between 200 and 400 PixelCoin".to_string(),
        true,
    );
    amount_term.add_precondition(Condition::AmountCondition {
        min: Some(200.0),
        max: Some(400.0),
    });
    custom_contract.add_term(amount_term);

    println!("Custom contract created with {} terms", custom_contract.terms.len());
    for term in &custom_contract.terms {
        println!("  - {}: {} preconditions, {} postconditions",
            term.name,
            term.preconditions.len(),
            term.postconditions.len()
        );
    }
    println!();

    // Demo 9: Contract lifecycle
    println!("--- Demo 9: Complete Contract Lifecycle ---");
    let mut lifecycle_contract = SmartContract::new(
        ContractType::Service,
        consumer_id,
        provider_id,
        150.0,
    );

    println!("1. Created - Status: {:?}", lifecycle_contract.status);

    lifecycle_contract.sign();
    println!("2. Signed - Status: {:?}", lifecycle_contract.status);

    lifecycle_contract.start_execution();
    println!("3. Executing - Status: {:?}", lifecycle_contract.status);

    lifecycle_contract.complete();
    println!("4. Completed - Status: {:?}", lifecycle_contract.status);
    println!("   Completed at: {:?}", lifecycle_contract.completed_at);
    println!();

    println!("=== Demo Complete ===");

    Ok(())
}
