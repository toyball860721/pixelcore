use super::*;
use uuid::Uuid;

// JWT 测试
#[test]
fn test_jwt_generate_and_verify() {
    let jwt_manager = JwtManager::default();
    let user_id = Uuid::new_v4();
    let roles = vec!["admin".to_string(), "user".to_string()];

    // 生成 token
    let token = jwt_manager
        .generate_token(user_id, None, roles.clone(), 3600)
        .unwrap();

    // 验证 token
    let claims = jwt_manager.verify_token(&token).unwrap();
    assert_eq!(claims.user_id, user_id);
    assert_eq!(claims.roles, roles);
    assert!(!claims.is_expired());
}

#[test]
fn test_jwt_expired_token() {
    let jwt_manager = JwtManager::default();
    let user_id = Uuid::new_v4();

    // 生成已过期的 token（-1 秒）
    let token = jwt_manager
        .generate_token(user_id, None, vec![], -1)
        .unwrap();

    // 验证应该失败
    let result = jwt_manager.verify_token(&token);
    assert!(result.is_err());
}

#[test]
fn test_jwt_refresh_token() {
    let jwt_manager = JwtManager::default();
    let user_id = Uuid::new_v4();

    // 生成原始 token
    let original_token = jwt_manager
        .generate_token(user_id, None, vec![], 3600)
        .unwrap();

    // 刷新 token
    let new_token = jwt_manager.refresh_token(&original_token, 3600).unwrap();

    // 验证新 token
    let claims = jwt_manager.verify_token(&new_token).unwrap();
    assert_eq!(claims.user_id, user_id);
}

#[test]
fn test_jwt_extract_user_id() {
    let jwt_manager = JwtManager::default();
    let user_id = Uuid::new_v4();

    let token = jwt_manager
        .generate_token(user_id, None, vec![], -1)
        .unwrap();

    // 即使 token 过期，也应该能提取 user_id
    let extracted_id = jwt_manager.extract_user_id(&token).unwrap();
    assert_eq!(extracted_id, user_id);
}

// API Key 测试
#[test]
fn test_api_key_create_and_verify() {
    let manager = ApiKeyManager::new();
    let user_id = Uuid::new_v4();
    let scopes = vec!["read".to_string(), "write".to_string()];

    // 创建 API Key
    let api_key = manager
        .create_key(user_id, "Test Key".to_string(), scopes.clone())
        .unwrap();

    assert_eq!(api_key.user_id, user_id);
    assert_eq!(api_key.scopes, scopes);
    assert!(api_key.is_valid());

    // 验证 API Key
    let verified = manager.verify_key(&api_key.key).unwrap();
    assert_eq!(verified.id, api_key.id);
}

#[test]
fn test_api_key_scope_check() {
    let manager = ApiKeyManager::new();
    let user_id = Uuid::new_v4();
    let scopes = vec!["read".to_string()];

    let api_key = manager
        .create_key(user_id, "Test Key".to_string(), scopes)
        .unwrap();

    // 应该有 read scope
    assert!(manager.check_scope(&api_key.key, "read").is_ok());

    // 不应该有 write scope
    assert!(manager.check_scope(&api_key.key, "write").is_err());
}

#[test]
fn test_api_key_revoke() {
    let manager = ApiKeyManager::new();
    let user_id = Uuid::new_v4();

    let api_key = manager
        .create_key(user_id, "Test Key".to_string(), vec![])
        .unwrap();

    // 撤销 API Key
    manager.revoke_key(&api_key.key).unwrap();

    // 验证应该失败
    let result = manager.verify_key(&api_key.key);
    assert!(result.is_err());
}

#[test]
fn test_api_key_get_user_keys() {
    let manager = ApiKeyManager::new();
    let user_id = Uuid::new_v4();

    // 创建多个 API Keys
    manager
        .create_key(user_id, "Key 1".to_string(), vec![])
        .unwrap();
    manager
        .create_key(user_id, "Key 2".to_string(), vec![])
        .unwrap();

    let user_keys = manager.get_user_keys(user_id);
    assert_eq!(user_keys.len(), 2);
}

// 加密测试
#[test]
fn test_encryption_decrypt() {
    let key = DataEncryptor::generate_key();
    let encryptor = DataEncryptor::new(&key).unwrap();

    let plaintext = b"Hello, World!";
    let encrypted = encryptor.encrypt(plaintext).unwrap();
    let decrypted = encryptor.decrypt(&encrypted).unwrap();

    assert_eq!(plaintext, &decrypted[..]);
}

#[test]
fn test_encryption_string() {
    let key = DataEncryptor::generate_key();
    let encryptor = DataEncryptor::new(&key).unwrap();

    let plaintext = "Hello, World!";
    let encrypted = encryptor.encrypt_string(plaintext).unwrap();
    let decrypted = encryptor.decrypt_string(&encrypted).unwrap();

    assert_eq!(plaintext, decrypted);
}

#[test]
fn test_encryption_invalid_key_length() {
    let key = vec![0u8; 16]; // 错误的密钥长度
    let result = DataEncryptor::new(&key);
    assert!(result.is_err());
}

#[test]
fn test_password_hashing() {
    let password = "my_secure_password";
    let salt = PasswordHasher::generate_salt();

    let hash = PasswordHasher::hash_password(password, &salt);

    // 验证正确的密码
    assert!(PasswordHasher::verify_password(password, &salt, &hash));

    // 验证错误的密码
    assert!(!PasswordHasher::verify_password("wrong_password", &salt, &hash));
}

// 密钥管理测试
#[test]
fn test_key_manager_generate_key() {
    let manager = KeyManager::new(90);

    let key_id = manager.generate_key().unwrap();
    let key = manager.get_key(key_id).unwrap();

    assert_eq!(key.id, key_id);
    assert_eq!(key.key.len(), 32);
}

#[test]
fn test_key_manager_active_key() {
    let manager = KeyManager::new(90);

    manager.generate_key().unwrap();
    let active_key = manager.get_active_key().unwrap();

    assert_eq!(active_key.key.len(), 32);
}

#[test]
fn test_key_manager_rotation() {
    let manager = KeyManager::new(90);

    let first_key_id = manager.generate_key().unwrap();
    let second_key_id = manager.rotate_key().unwrap();

    assert_ne!(first_key_id, second_key_id);

    // 活跃密钥应该是第二个
    let active_key = manager.get_active_key().unwrap();
    assert_eq!(active_key.id, second_key_id);

    // 第一个密钥应该有轮换时间
    let first_key = manager.get_key(first_key_id).unwrap();
    assert!(first_key.rotated_at.is_some());
}

#[test]
fn test_key_manager_should_rotate() {
    let manager = KeyManager::new(0); // 0 天轮换间隔

    manager.generate_key().unwrap();

    // 应该需要轮换
    assert!(manager.should_rotate());
}

#[test]
fn test_key_manager_cleanup() {
    let manager = KeyManager::new(90);

    // 生成 5 个密钥
    for _ in 0..5 {
        manager.generate_key().unwrap();
    }

    assert_eq!(manager.list_keys().len(), 5);

    // 只保留最近的 2 个（但活跃密钥总是会被保留，所以可能是 2 或 3 个）
    manager.cleanup_old_keys(2);

    // 应该保留 2-3 个密钥（取决于活跃密钥是否在最近 2 个中）
    let remaining = manager.list_keys().len();
    assert!(remaining >= 2 && remaining <= 3);
}

// 安全审计测试
#[test]
fn test_security_auditor_log() {
    let auditor = SecurityAuditor::new(100);
    let user_id = Uuid::new_v4();

    let log = SecurityAuditLog::new(SecurityEventType::LoginSuccess {
        user_id,
        method: AuthMethod::JwtToken,
    });

    auditor.log(log);

    assert_eq!(auditor.count(), 1);
}

#[test]
fn test_security_auditor_login_attempt() {
    let auditor = SecurityAuditor::new(100);
    let user_id = Uuid::new_v4();

    // 记录成功登录
    auditor.log_login_attempt("user@example.com", true, Some(user_id), None);

    let logs = auditor.get_all_logs();
    assert!(logs.len() > 0);
}

#[test]
fn test_security_auditor_brute_force_detection() {
    let auditor = SecurityAuditor::new(100);

    // 模拟 5 次失败登录
    for _ in 0..5 {
        auditor.log_login_attempt("user@example.com", false, None, None);
    }

    // 应该检测到暴力破解
    let critical_events = auditor.get_critical_events();
    assert!(critical_events.len() > 0);
}

#[test]
fn test_security_auditor_search() {
    let auditor = SecurityAuditor::new(100);
    let user_id = Uuid::new_v4();

    auditor.log(SecurityAuditLog::new(SecurityEventType::LoginSuccess {
        user_id,
        method: AuthMethod::JwtToken,
    }));

    auditor.log(SecurityAuditLog::new(SecurityEventType::LoginFailure {
        username: "test".to_string(),
        reason: "Invalid".to_string(),
        method: AuthMethod::Password,
    }));

    let failed_logins = auditor.get_failed_logins();
    assert_eq!(failed_logins.len(), 1);
}

#[test]
fn test_security_auditor_stats() {
    let auditor = SecurityAuditor::new(100);
    let user_id = Uuid::new_v4();

    // 记录一些事件
    auditor.log_login_attempt("user@example.com", true, Some(user_id), None);
    auditor.log_login_attempt("user@example.com", false, None, None);

    let stats = auditor.get_stats();
    assert!(stats.total_logs > 0);
}

#[test]
fn test_security_auditor_cleanup() {
    let auditor = SecurityAuditor::new(100);

    // 记录一些登录尝试
    for _ in 0..10 {
        auditor.log_login_attempt("user@example.com", false, None, None);
    }

    // 清理旧记录
    auditor.cleanup_old_attempts();
    auditor.cleanup_old_accesses();

    // 应该仍然有记录（因为是最近的）
    assert!(auditor.count() > 0);
}
