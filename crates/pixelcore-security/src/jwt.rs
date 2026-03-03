use crate::models::JwtClaims;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum JwtError {
    #[error("Failed to encode JWT: {0}")]
    EncodingError(String),
    #[error("Failed to decode JWT: {0}")]
    DecodingError(String),
    #[error("Token expired")]
    TokenExpired,
    #[error("Invalid token")]
    InvalidToken,
}

pub type JwtResult<T> = Result<T, JwtError>;

/// JWT Token 管理器
pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    algorithm: Algorithm,
}

impl JwtManager {
    /// 创建新的 JWT 管理器（使用 HS256 算法）
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
            algorithm: Algorithm::HS256,
        }
    }

    /// 生成 JWT token
    pub fn generate_token(
        &self,
        user_id: Uuid,
        tenant_id: Option<Uuid>,
        roles: Vec<String>,
        expires_in_seconds: i64,
    ) -> JwtResult<String> {
        let claims = JwtClaims::new(user_id, tenant_id, roles, expires_in_seconds);

        encode(&Header::new(self.algorithm), &claims, &self.encoding_key)
            .map_err(|e| JwtError::EncodingError(e.to_string()))
    }

    /// 验证并解码 JWT token
    pub fn verify_token(&self, token: &str) -> JwtResult<JwtClaims> {
        let mut validation = Validation::new(self.algorithm);
        validation.set_issuer(&["pixelcore"]);
        validation.set_audience(&["pixelcore-api"]);

        let token_data = decode::<JwtClaims>(token, &self.decoding_key, &validation)
            .map_err(|e| JwtError::DecodingError(e.to_string()))?;

        let claims = token_data.claims;

        if claims.is_expired() {
            return Err(JwtError::TokenExpired);
        }

        Ok(claims)
    }

    /// 刷新 token（生成新的 token）
    pub fn refresh_token(&self, old_token: &str, expires_in_seconds: i64) -> JwtResult<String> {
        let claims = self.verify_token(old_token)?;

        self.generate_token(
            claims.user_id,
            claims.tenant_id,
            claims.roles,
            expires_in_seconds,
        )
    }

    /// 从 token 中提取用户 ID（不验证过期时间）
    pub fn extract_user_id(&self, token: &str) -> JwtResult<Uuid> {
        let mut validation = Validation::new(self.algorithm);
        validation.validate_exp = false; // 不验证过期时间
        validation.set_issuer(&["pixelcore"]);
        validation.set_audience(&["pixelcore-api"]);

        let token_data = decode::<JwtClaims>(token, &self.decoding_key, &validation)
            .map_err(|e| JwtError::DecodingError(e.to_string()))?;

        Ok(token_data.claims.user_id)
    }
}

impl Default for JwtManager {
    fn default() -> Self {
        // 默认使用一个固定的密钥（生产环境应该使用环境变量）
        Self::new(b"pixelcore-default-secret-key-change-in-production")
    }
}
