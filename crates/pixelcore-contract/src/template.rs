use crate::models::{SmartContract, ContractType, ContractTerm, Condition};
use uuid::Uuid;
use chrono::{Duration, Utc};

pub struct ContractTemplate;

impl ContractTemplate {
    /// 创建服务合约模板
    pub fn service_contract(
        party_a: Uuid,
        party_b: Uuid,
        service_name: String,
        amount: f64,
        duration: Duration,
    ) -> SmartContract {
        let mut contract = SmartContract::new(ContractType::Service, party_a, party_b, amount);

        contract.end_time = Some(Utc::now() + duration);

        // 服务交付条款
        let mut delivery_term = ContractTerm::new(
            "Service Delivery".to_string(),
            format!("Provider must deliver {} within specified timeframe", service_name),
            true,
        );
        delivery_term.add_precondition(Condition::StatusCondition {
            required_status: "executing".to_string(),
        });
        delivery_term.add_postcondition(Condition::StatusCondition {
            required_status: "completed".to_string(),
        });
        contract.add_term(delivery_term);

        // 付款条款
        let mut payment_term = ContractTerm::new(
            "Payment".to_string(),
            format!("Consumer must pay {} PixelCoin upon service completion", amount),
            true,
        );
        payment_term.add_precondition(Condition::AmountCondition {
            min: Some(amount),
            max: None,
        });
        contract.add_term(payment_term);

        contract
    }

    /// 创建数据购买合约模板
    pub fn data_purchase_contract(
        party_a: Uuid,
        party_b: Uuid,
        data_name: String,
        amount: f64,
    ) -> SmartContract {
        let mut contract = SmartContract::new(ContractType::Data, party_a, party_b, amount);

        // 数据交付条款
        let mut delivery_term = ContractTerm::new(
            "Data Delivery".to_string(),
            format!("Provider must deliver {} dataset", data_name),
            true,
        );
        delivery_term.add_precondition(Condition::StatusCondition {
            required_status: "confirmed".to_string(),
        });
        contract.add_term(delivery_term);

        // 付款条款
        let mut payment_term = ContractTerm::new(
            "Payment".to_string(),
            format!("Buyer must pay {} PixelCoin", amount),
            true,
        );
        payment_term.add_precondition(Condition::AmountCondition {
            min: Some(amount),
            max: Some(amount),
        });
        contract.add_term(payment_term);

        contract
    }

    /// 创建订阅合约模板
    pub fn subscription_contract(
        party_a: Uuid,
        party_b: Uuid,
        service_name: String,
        monthly_amount: f64,
        duration: Duration,
    ) -> SmartContract {
        let mut contract = SmartContract::new(ContractType::Subscription, party_a, party_b, monthly_amount);

        let end_time = Utc::now() + duration;
        contract.end_time = Some(end_time);

        // 服务访问条款
        let mut access_term = ContractTerm::new(
            "Service Access".to_string(),
            format!("Provider grants access to {} for subscription period", service_name),
            true,
        );
        access_term.add_precondition(Condition::TimeCondition {
            before: Some(end_time),
            after: None,
        });
        contract.add_term(access_term);

        // 定期付款条款
        let mut payment_term = ContractTerm::new(
            "Monthly Payment".to_string(),
            format!("Subscriber must pay {} PixelCoin monthly", monthly_amount),
            true,
        );
        payment_term.add_precondition(Condition::AmountCondition {
            min: Some(monthly_amount),
            max: None,
        });
        contract.add_term(payment_term);

        contract
    }

    /// 创建计算资源合约模板
    pub fn compute_contract(
        party_a: Uuid,
        party_b: Uuid,
        resource_type: String,
        amount: f64,
        compute_hours: u32,
    ) -> SmartContract {
        let mut contract = SmartContract::new(ContractType::Compute, party_a, party_b, amount);

        // 资源提供条款
        let mut resource_term = ContractTerm::new(
            "Resource Provision".to_string(),
            format!("Provider must provide {} for {} hours", resource_type, compute_hours),
            true,
        );
        resource_term.add_precondition(Condition::StatusCondition {
            required_status: "executing".to_string(),
        });
        contract.add_term(resource_term);

        // 付款条款
        let mut payment_term = ContractTerm::new(
            "Payment".to_string(),
            format!("Consumer must pay {} PixelCoin for compute resources", amount),
            true,
        );
        payment_term.add_precondition(Condition::AmountCondition {
            min: Some(amount),
            max: None,
        });
        contract.add_term(payment_term);

        contract
    }
}
