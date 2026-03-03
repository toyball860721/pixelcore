use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Subject (user ID)
    pub sub: String,
    /// Issued at
    pub iat: i64,
    /// Expiration time
    pub exp: i64,
    /// Issuer
    pub iss: String,
    /// Audience
    pub aud: String,
    /// Custom claims
    pub user_id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub roles: Vec<String>,
}

impl JwtClaims {
    pub fn new(user_id: Uuid, tenant_id: Option<Uuid>, roles: Vec<String>, expires_in_seconds: i64) -> Self {
        let now = Utc::now().timestamp();
        Self {
            sub: user_id.to_string(),
            iat: now,
            exp: now + expires_in_seconds,
            iss: "pixelcore".to_string(),
            aud: "pixelcore-api".to_string(),
            user_id,
            tenant_id,
            roles,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }
}

/// API Key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub key: String,
    pub user_id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub name: String,
    pub scopes: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

impl ApiKey {
    pub fn new(user_id: Uuid, name: String, scopes: Vec<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            key: Self::generate_key(),
            user_id,
            tenant_id: None,
            name,
            scopes,
            created_at: Utc::now(),
            expires_at: None,
            last_used_at: None,
            is_active: true,
        }
    }

    pub fn with_tenant(mut self, tenant_id: Uuid) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    pub fn with_expiry(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    pub fn is_valid(&self) -> bool {
        self.is_active && !self.is_expired()
    }

    fn generate_key() -> String {
        use rand::Rng;
        let random_bytes: Vec<u8> = (0..32).map(|_| rand::thread_rng().gen()).collect();
        format!("pk_{}", base64::encode(&random_bytes))
    }
}

/// OAuth Token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

/// 加密密钥
#[derive(Debug, Clone)]
pub struct EncryptionKey {
    pub id: Uuid,
    pub key: Vec<u8>,
    pub algorithm: EncryptionAlgorithm,
    pub created_at: DateTime<Utc>,
    pub rotated_at: Option<DateTime<Utc>>,
}

impl EncryptionKey {
    pub fn new_aes256() -> Self {
        use rand::Rng;
        let key: Vec<u8> = (0..32).map(|_| rand::thread_rng().gen()).collect();
        Self {
            id: Uuid::new_v4(),
            key,
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            created_at: Utc::now(),
            rotated_at: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
}

/// 安全审计事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventType {
    /// 登录成功
    LoginSuccess {
        user_id: Uuid,
        method: AuthMethod,
    },
    /// 登录失败
    LoginFailure {
        username: String,
        reason: String,
        method: AuthMethod,
    },
    /// 登出
    Logout {
        user_id: Uuid,
    },
    /// Token 刷新
    TokenRefresh {
        user_id: Uuid,
    },
    /// API Key 创建
    ApiKeyCreated {
        key_id: Uuid,
        user_id: Uuid,
    },
    /// API Key 撤销
    ApiKeyRevoked {
        key_id: Uuid,
        user_id: Uuid,
    },
    /// 访问被拒绝
    AccessDenied {
        user_id: Uuid,
        resource: String,
        reason: String,
    },
    /// 异常活动检测
    AnomalousActivity {
        user_id: Uuid,
        description: String,
        severity: SecuritySeverity,
    },
    /// 密钥轮换
    KeyRotation {
        key_id: Uuid,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AuthMethod {
    JwtToken,
    ApiKey,
    OAuth,
    Password,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// 安全审计日志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAuditLog {
    pub id: Uuid,
    pub event_type: SecurityEventType,
    pub timestamp: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

impl SecurityAuditLog {
    pub fn new(event_type: SecurityEventType) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            timestamp: Utc::now(),
            ip_address: None,
            user_agent: None,
            metadata: None,
        }
    }

    pub fn with_ip(mut self, ip: String) -> Self {
        self.ip_address = Some(ip);
        self
    }

    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}
