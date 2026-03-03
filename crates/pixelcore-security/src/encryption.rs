use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::Rng;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EncryptionError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("Invalid key length")]
    InvalidKeyLength,
    #[error("Invalid nonce length")]
    InvalidNonceLength,
}

pub type EncryptionResult<T> = Result<T, EncryptionError>;

/// 数据加密器（AES-256-GCM）
pub struct DataEncryptor {
    cipher: Aes256Gcm,
}

impl DataEncryptor {
    /// 使用指定的密钥创建加密器
    pub fn new(key: &[u8]) -> EncryptionResult<Self> {
        if key.len() != 32 {
            return Err(EncryptionError::InvalidKeyLength);
        }

        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;

        Ok(Self { cipher })
    }

    /// 生成随机密钥（32 字节）
    pub fn generate_key() -> Vec<u8> {
        let mut key = vec![0u8; 32];
        rand::thread_rng().fill(&mut key[..]);
        key
    }

    /// 加密数据
    pub fn encrypt(&self, plaintext: &[u8]) -> EncryptionResult<Vec<u8>> {
        // 生成随机 nonce (12 字节)
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // 加密数据
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;

        // 将 nonce 和 ciphertext 组合在一起
        // 格式: [nonce (12 bytes)][ciphertext]
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// 解密数据
    pub fn decrypt(&self, encrypted_data: &[u8]) -> EncryptionResult<Vec<u8>> {
        if encrypted_data.len() < 12 {
            return Err(EncryptionError::InvalidNonceLength);
        }

        // 分离 nonce 和 ciphertext
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        // 解密数据
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))?;

        Ok(plaintext)
    }

    /// 加密字符串
    pub fn encrypt_string(&self, plaintext: &str) -> EncryptionResult<String> {
        let encrypted = self.encrypt(plaintext.as_bytes())?;
        Ok(base64::encode(&encrypted))
    }

    /// 解密字符串
    pub fn decrypt_string(&self, encrypted_base64: &str) -> EncryptionResult<String> {
        let encrypted = base64::decode(encrypted_base64)
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))?;

        let decrypted = self.decrypt(&encrypted)?;

        String::from_utf8(decrypted)
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))
    }
}

/// 密码哈希器（使用 SHA-256）
pub struct PasswordHasher;

impl PasswordHasher {
    /// 哈希密码
    pub fn hash_password(password: &str, salt: &[u8]) -> String {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(salt);
        let result = hasher.finalize();

        base64::encode(&result)
    }

    /// 生成随机盐
    pub fn generate_salt() -> Vec<u8> {
        let mut salt = vec![0u8; 16];
        rand::thread_rng().fill(&mut salt[..]);
        salt
    }

    /// 验证密码
    pub fn verify_password(password: &str, salt: &[u8], hash: &str) -> bool {
        let computed_hash = Self::hash_password(password, salt);
        computed_hash == hash
    }
}
