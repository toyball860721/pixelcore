use crate::models::{SmartContract, ContractStatus, Condition, ContractExecutionResult};
use pixelcore_transaction::{Transaction, TransactionStatus};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Clone)]
pub struct ContractExecutor {
    contracts: Arc<Mutex<Vec<SmartContract>>>,
}

impl ContractExecutor {
    pub fn new() -> Self {
        Self {
            contracts: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Register a contract for execution
    pub async fn register_contract(&self, contract: SmartContract) -> Result<(), String> {
        if contract.status != ContractStatus::Active {
            return Err("Contract must be active to register for execution".to_string());
        }

        let mut contracts = self.contracts.lock().await;
        contracts.push(contract);
        Ok(())
    }

    /// Execute a contract based on transaction state
    pub async fn execute_contract(
        &self,
        contract_id: Uuid,
        transaction: &Transaction,
    ) -> Result<ContractExecutionResult, String> {
        let mut contracts = self.contracts.lock().await;

        let contract = contracts
            .iter_mut()
            .find(|c| c.id == contract_id)
            .ok_or_else(|| format!("Contract {} not found", contract_id))?;

        // Check if contract is in executable state
        if contract.status != ContractStatus::Active && contract.status != ContractStatus::Executing {
            return Err(format!("Contract is in {:?} state, cannot execute", contract.status));
        }

        // Start execution if not already executing
        if contract.status == ContractStatus::Active {
            contract.start_execution();
        }

        // Build context from transaction
        let context = serde_json::json!({
            "amount": transaction.amount,
            "status": format!("{:?}", transaction.status).to_lowercase(),
            "transaction_id": transaction.id,
        });

        // Check all preconditions
        let preconditions_met = contract.validate_preconditions(&context);

        if !preconditions_met {
            return Ok(ContractExecutionResult {
                contract_id: contract.id,
                success: false,
                result: None,
                error: Some("Preconditions not met".to_string()),
                executed_at: Utc::now(),
            });
        }

        // Execute contract terms
        self.execute_terms(contract, transaction).await?;

        // Check if contract should be completed
        if self.should_complete(contract, transaction).await? {
            contract.complete();
        }

        Ok(ContractExecutionResult {
            contract_id: contract.id,
            success: true,
            result: Some(serde_json::json!({"status": "executed"})),
            error: None,
            executed_at: Utc::now(),
        })
    }

    /// Execute contract terms
    async fn execute_terms(
        &self,
        contract: &SmartContract,
        _transaction: &Transaction,
    ) -> Result<(), String> {
        for term in &contract.terms {
            // Log term execution
            println!(
                "Executing term: {} for contract {}",
                term.name, contract.id
            );

            // In a real system, this would trigger actual actions
            // For now, we just validate and log
        }
        Ok(())
    }

    /// Check if contract should be completed
    async fn should_complete(
        &self,
        _contract: &SmartContract,
        transaction: &Transaction,
    ) -> Result<bool, String> {
        // Contract completes when transaction is completed
        Ok(transaction.status == TransactionStatus::Completed)
    }

    /// Get all registered contracts
    pub async fn get_contracts(&self) -> Vec<SmartContract> {
        let contracts = self.contracts.lock().await;
        contracts.clone()
    }

    /// Get contract by ID
    pub async fn get_contract(&self, contract_id: Uuid) -> Option<SmartContract> {
        let contracts = self.contracts.lock().await;
        contracts.iter().find(|c| c.id == contract_id).cloned()
    }

    /// Remove completed or terminated contracts
    pub async fn cleanup_contracts(&self) -> usize {
        let mut contracts = self.contracts.lock().await;
        let initial_count = contracts.len();
        contracts.retain(|c| {
            c.status != ContractStatus::Completed && c.status != ContractStatus::Terminated
        });
        initial_count - contracts.len()
    }
}
