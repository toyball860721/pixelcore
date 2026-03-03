pub mod api_key;
pub mod encryption;
pub mod jwt;
pub mod key_manager;
pub mod models;
pub mod security_audit;

#[cfg(test)]
mod tests;

// Re-exports
pub use api_key::{ApiKeyError, ApiKeyManager, ApiKeyResult};
pub use encryption::{DataEncryptor, EncryptionError, EncryptionResult, PasswordHasher};
pub use jwt::{JwtError, JwtManager, JwtResult};
pub use key_manager::{KeyManager, KeyManagerError, KeyManagerResult, KeyStats};
pub use models::{
    ApiKey, AuthMethod, EncryptionAlgorithm, EncryptionKey, JwtClaims, OAuthToken,
    SecurityAuditLog, SecurityEventType, SecuritySeverity,
};
pub use security_audit::{SecurityAuditor, SecurityStats};
