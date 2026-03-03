use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// 合约类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContractType {
    /// 服务合约
    Service,
    /// 数据合约
    Data,
    /// 计算合约
    Compute,
    /// 订阅合约
    Subscription,
}

/// 合约状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContractStatus {
    /// 草稿
    Draft,
    /// 待签署
    PendingSignature,
    /// 已激活
    Active,
    /// 执行中
    Executing,
    /// 已完成
    Completed,
    /// 已终止
    Terminated,
    /// 争议中
    Disputed,
}

/// 条件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    /// 时间条件 (在指定时间之前/之后)
    TimeCondition {
        before: Option<DateTime<Utc>>,
        after: Option<DateTime<Utc>>,
    },
    /// 金额条件
    AmountCondition {
        min: Option<f64>,
        max: Option<f64>,
    },
    /// 状态条件
    StatusCondition {
        required_status: String,
    },
    /// 自定义条件 (表达式)
    CustomCondition {
        expression: String,
    },
}

impl Condition {
    /// 检查条件是否满足
    pub fn check(&self, context: &serde_json::Value) -> bool {
        match self {
            Condition::TimeCondition { before, after } => {
                let now = Utc::now();
                if let Some(before_time) = before {
                    if now > *before_time {
                        return false;
                    }
                }
                if let Some(after_time) = after {
                    if now < *after_time {
                        return false;
                    }
                }
                true
            }
            Condition::AmountCondition { min, max } => {
                if let Some(amount) = context.get("amount").and_then(|v| v.as_f64()) {
                    if let Some(min_amount) = min {
                        if amount < *min_amount {
                            return false;
                        }
                    }
                    if let Some(max_amount) = max {
                        if amount > *max_amount {
                            return false;
                        }
                    }
                    true
                } else {
                    false
                }
            }
            Condition::StatusCondition { required_status } => {
                if let Some(status) = context.get("status").and_then(|v| v.as_str()) {
                    status == required_status
                } else {
                    false
                }
            }
            Condition::CustomCondition { expression: _ } => {
                // 简化实现: 自定义条件总是返回 true
                // 实际应该实现表达式解析和求值
                true
            }
        }
    }
}

/// 合约条款
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractTerm {
    /// 条款 ID
    pub id: Uuid,
    /// 条款名称
    pub name: String,
    /// 条款描述
    pub description: String,
    /// 前置条件
    pub preconditions: Vec<Condition>,
    /// 后置条件
    pub postconditions: Vec<Condition>,
    /// 是否必需
    pub required: bool,
}

impl ContractTerm {
    /// 创建新条款
    pub fn new(name: String, description: String, required: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            preconditions: Vec::new(),
            postconditions: Vec::new(),
            required,
        }
    }

    /// 添加前置条件
    pub fn add_precondition(&mut self, condition: Condition) {
        self.preconditions.push(condition);
    }

    /// 添加后置条件
    pub fn add_postcondition(&mut self, condition: Condition) {
        self.postconditions.push(condition);
    }

    /// 验证前置条件
    pub fn validate_preconditions(&self, context: &serde_json::Value) -> bool {
        self.preconditions.iter().all(|c| c.check(context))
    }

    /// 验证后置条件
    pub fn validate_postconditions(&self, context: &serde_json::Value) -> bool {
        self.postconditions.iter().all(|c| c.check(context))
    }
}

/// 智能合约
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartContract {
    /// 合约 ID
    pub id: Uuid,
    /// 合约类型
    pub contract_type: ContractType,
    /// 合约状态
    pub status: ContractStatus,
    /// 甲方 (买方)
    pub party_a: Uuid,
    /// 乙方 (卖方)
    pub party_b: Uuid,
    /// 合约条款
    pub terms: Vec<ContractTerm>,
    /// 合约金额
    pub amount: f64,
    /// 货币单位
    pub currency: String,
    /// 开始时间
    pub start_time: Option<DateTime<Utc>>,
    /// 结束时间
    pub end_time: Option<DateTime<Utc>>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 签署时间
    pub signed_at: Option<DateTime<Utc>>,
    /// 完成时间
    pub completed_at: Option<DateTime<Utc>>,
    /// 元数据
    pub metadata: serde_json::Value,
}

impl SmartContract {
    /// 创建新合约
    pub fn new(
        contract_type: ContractType,
        party_a: Uuid,
        party_b: Uuid,
        amount: f64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            contract_type,
            status: ContractStatus::Draft,
            party_a,
            party_b,
            terms: Vec::new(),
            amount,
            currency: "PixelCoin".to_string(),
            start_time: None,
            end_time: None,
            created_at: Utc::now(),
            signed_at: None,
            completed_at: None,
            metadata: serde_json::json!({}),
        }
    }

    /// 添加条款
    pub fn add_term(&mut self, term: ContractTerm) {
        self.terms.push(term);
    }

    /// 签署合约
    pub fn sign(&mut self) {
        self.status = ContractStatus::Active;
        self.signed_at = Some(Utc::now());
        self.start_time = Some(Utc::now());
    }

    /// 激活合约
    pub fn activate(&mut self) {
        self.status = ContractStatus::Active;
    }

    /// 开始执行
    pub fn start_execution(&mut self) {
        self.status = ContractStatus::Executing;
    }

    /// 完成合约
    pub fn complete(&mut self) {
        self.status = ContractStatus::Completed;
        self.completed_at = Some(Utc::now());
    }

    /// 终止合约
    pub fn terminate(&mut self) {
        self.status = ContractStatus::Terminated;
        self.completed_at = Some(Utc::now());
    }

    /// 发起争议
    pub fn dispute(&mut self) {
        self.status = ContractStatus::Disputed;
    }

    /// 验证所有必需条款的前置条件
    pub fn validate_preconditions(&self, context: &serde_json::Value) -> bool {
        self.terms
            .iter()
            .filter(|t| t.required)
            .all(|t| t.validate_preconditions(context))
    }

    /// 验证所有必需条款的后置条件
    pub fn validate_postconditions(&self, context: &serde_json::Value) -> bool {
        self.terms
            .iter()
            .filter(|t| t.required)
            .all(|t| t.validate_postconditions(context))
    }

    /// 检查合约是否过期
    pub fn is_expired(&self) -> bool {
        if let Some(end_time) = self.end_time {
            Utc::now() > end_time
        } else {
            false
        }
    }
}

/// 合约执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractExecutionResult {
    /// 合约 ID
    pub contract_id: Uuid,
    /// 是否成功
    pub success: bool,
    /// 执行结果
    pub result: Option<serde_json::Value>,
    /// 错误信息
    pub error: Option<String>,
    /// 执行时间
    pub executed_at: DateTime<Utc>,
}
