use crate::models::{SmartContract, ContractStatus};

#[derive(Debug, Clone)]
pub struct ContractValidator;

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.is_valid = false;
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

impl ContractValidator {
    pub fn new() -> Self {
        Self
    }

    /// Validate a complete contract
    pub fn validate_contract(&self, contract: &SmartContract) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Validate basic fields
        self.validate_basic_fields(contract, &mut result);

        // Validate parties
        self.validate_parties(contract, &mut result);

        // Validate terms
        self.validate_terms(contract, &mut result);

        // Validate status consistency
        self.validate_status(contract, &mut result);

        // Validate timestamps
        self.validate_timestamps(contract, &mut result);

        result
    }

    /// Validate basic contract fields
    fn validate_basic_fields(&self, contract: &SmartContract, result: &mut ValidationResult) {
        if contract.terms.is_empty() {
            result.add_warning("Contract has no terms".to_string());
        }

        if contract.amount < 0.0 {
            result.add_error("Contract amount cannot be negative".to_string());
        }
    }

    /// Validate contract parties
    fn validate_parties(&self, contract: &SmartContract, result: &mut ValidationResult) {
        if contract.party_a == contract.party_b {
            result.add_error("Party A and Party B cannot be the same".to_string());
        }
    }

    /// Validate contract terms
    fn validate_terms(&self, contract: &SmartContract, result: &mut ValidationResult) {
        for (index, term) in contract.terms.iter().enumerate() {
            if term.name.is_empty() {
                result.add_error(format!("Term {} has empty name", index));
            }

            if term.description.is_empty() {
                result.add_error(format!("Term {} has empty description", index));
            }

            if term.preconditions.is_empty() && term.postconditions.is_empty() {
                result.add_warning(format!("Term {} has no conditions", index));
            }
        }
    }

    /// Validate contract status
    fn validate_status(&self, contract: &SmartContract, result: &mut ValidationResult) {
        match contract.status {
            ContractStatus::Active | ContractStatus::Executing => {
                if contract.signed_at.is_none() {
                    result.add_error("Active/Executing contract must have signed_at timestamp".to_string());
                }
            }
            ContractStatus::Completed => {
                if contract.signed_at.is_none() {
                    result.add_error("Completed contract must have signed_at timestamp".to_string());
                }
                if contract.completed_at.is_none() {
                    result.add_error("Completed contract must have completed_at timestamp".to_string());
                }
            }
            _ => {}
        }
    }

    /// Validate contract timestamps
    fn validate_timestamps(&self, contract: &SmartContract, result: &mut ValidationResult) {
        let now = chrono::Utc::now();

        // Check created_at is not in the future
        if contract.created_at > now {
            result.add_error("Contract creation time cannot be in the future".to_string());
        }

        // Check signed_at is after created_at
        if let Some(signed) = contract.signed_at {
            if signed < contract.created_at {
                result.add_error("Signed time cannot be before creation time".to_string());
            }
        }

        // Check completed_at is after signed_at
        if let (Some(completed), Some(signed)) = (contract.completed_at, contract.signed_at) {
            if completed < signed {
                result.add_error("Completion time cannot be before signed time".to_string());
            }
        }

        // Check start_time and end_time
        if let (Some(start), Some(end)) = (contract.start_time, contract.end_time) {
            if end <= start {
                result.add_error("End time must be after start time".to_string());
            }
        }
    }

    /// Quick validation check (returns true if valid)
    pub fn is_valid(&self, contract: &SmartContract) -> bool {
        self.validate_contract(contract).is_valid
    }

    /// Get validation errors only
    pub fn get_errors(&self, contract: &SmartContract) -> Vec<String> {
        self.validate_contract(contract).errors
    }

    /// Get validation warnings only
    pub fn get_warnings(&self, contract: &SmartContract) -> Vec<String> {
        self.validate_contract(contract).warnings
    }
}
