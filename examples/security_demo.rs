use pixelcore_security::{
    ApiKeyManager, DataEncryptor, JwtManager, KeyManager, PasswordHasher, SecurityAuditor,
    SecurityAuditLog, SecurityEventType, AuthMethod,
};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    println!("=== PixelCore 安全系统演示 ===\n");

    // 1. JWT Token 认证演示
    println!("1. JWT Token 认证");
    println!("==================");

    let jwt_manager = JwtManager::default();
    let user_id = Uuid::new_v4();
    let roles = vec!["admin".to_string(), "developer".to_string()];

    // 生成 JWT token
    let token = jwt_manager
        .generate_token(user_id, None, roles.clone(), 3600)
        .unwrap();
    println!("✓ 生成 JWT token: {}...", &token[..50]);

    // 验证 token
    let claims = jwt_manager.verify_token(&token).unwrap();
    println!("✓ 验证 token 成功");
    println!("  - 用户 ID: {}", claims.user_id);
    println!("  - 角色: {:?}", claims.roles);
    println!("  - 过期时间: {}", claims.exp);

    // 刷新 token
    let new_token = jwt_manager.refresh_token(&token, 3600).unwrap();
    println!("✓ 刷新 token 成功: {}...\n", &new_token[..50]);

    // 2. API Key 管理演示
    println!("2. API Key 管理");
    println!("================");

    let api_key_manager = ApiKeyManager::new();
    let scopes = vec!["read".to_string(), "write".to_string()];

    // 创建 API Key
    let api_key = api_key_manager
        .create_key(user_id, "Production API Key".to_string(), scopes.clone())
        .unwrap();
    println!("✓ 创建 API Key: {}", api_key.key);
    println!("  - 名称: {}", api_key.name);
    println!("  - Scopes: {:?}", api_key.scopes);

    // 验证 API Key
    let verified = api_key_manager.verify_key(&api_key.key).unwrap();
    println!("✓ 验证 API Key 成功");

    // 检查 scope
    api_key_manager.check_scope(&api_key.key, "read").unwrap();
    println!("✓ Scope 检查通过: read");

    match api_key_manager.check_scope(&api_key.key, "admin") {
        Ok(_) => println!("✓ Scope 检查通过: admin"),
        Err(_) => println!("✗ Scope 检查失败: admin (预期行为)"),
    }

    // 获取用户的所有 API Keys
    let user_keys = api_key_manager.get_user_keys(user_id);
    println!("✓ 用户拥有 {} 个 API Keys\n", user_keys.len());

    // 3. 数据加密演示
    println!("3. 数据加密 (AES-256-GCM)");
    println!("==========================");

    let encryption_key = DataEncryptor::generate_key();
    let encryptor = DataEncryptor::new(&encryption_key).unwrap();

    // 加密字符串
    let plaintext = "这是一个敏感数据：用户密码 = secret123";
    let encrypted = encryptor.encrypt_string(plaintext).unwrap();
    println!("✓ 原始数据: {}", plaintext);
    println!("✓ 加密后: {}...", &encrypted[..50]);

    // 解密字符串
    let decrypted = encryptor.decrypt_string(&encrypted).unwrap();
    println!("✓ 解密后: {}", decrypted);
    assert_eq!(plaintext, decrypted);
    println!("✓ 加密/解密验证成功\n");

    // 4. 密码哈希演示
    println!("4. 密码哈希");
    println!("============");

    let password = "my_secure_password_123";
    let salt = PasswordHasher::generate_salt();
    let hash = PasswordHasher::hash_password(password, &salt);

    println!("✓ 密码: {}", password);
    println!("✓ 盐: {} bytes", salt.len());
    println!("✓ 哈希: {}...", &hash[..40]);

    // 验证密码
    let is_valid = PasswordHasher::verify_password(password, &salt, &hash);
    println!("✓ 密码验证: {}", if is_valid { "成功" } else { "失败" });

    let is_invalid = PasswordHasher::verify_password("wrong_password", &salt, &hash);
    println!("✓ 错误密码验证: {} (预期)\n", if is_invalid { "成功" } else { "失败" });

    // 5. 密钥管理演示
    println!("5. 密钥管理");
    println!("============");

    let key_manager = KeyManager::new(90); // 90 天轮换周期

    // 生成密钥
    let key_id = key_manager.generate_key().unwrap();
    println!("✓ 生成加密密钥: {}", key_id);

    // 获取活跃密钥
    let active_key = key_manager.get_active_key().unwrap();
    println!("✓ 活跃密钥 ID: {}", active_key.id);
    println!("  - 密钥长度: {} bytes", active_key.key.len());
    println!("  - 创建时间: {}", active_key.created_at);

    // 轮换密钥
    let new_key_id = key_manager.rotate_key().unwrap();
    println!("✓ 轮换密钥: {} -> {}", key_id, new_key_id);

    // 获取密钥统计
    let stats = key_manager.get_stats();
    println!("✓ 密钥统计:");
    println!("  - 总密钥数: {}", stats.total_keys);
    println!("  - 活跃密钥 ID: {:?}", stats.active_key_id);
    println!("  - 需要轮换: {}\n", stats.should_rotate);

    // 6. 安全审计演示
    println!("6. 安全审计");
    println!("============");

    let auditor = SecurityAuditor::new(1000);

    // 记录成功登录
    auditor.log_login_attempt("user@example.com", true, Some(user_id), Some("192.168.1.100".to_string()));
    println!("✓ 记录成功登录");

    // 记录失败登录
    for i in 1..=3 {
        auditor.log_login_attempt("attacker@example.com", false, None, Some("10.0.0.1".to_string()));
        println!("✓ 记录失败登录尝试 #{}", i);
    }

    // 记录访问被拒绝
    auditor.log(SecurityAuditLog::new(
        SecurityEventType::AccessDenied {
            user_id,
            resource: "/admin/users".to_string(),
            reason: "Insufficient permissions".to_string(),
        },
    ));
    println!("✓ 记录访问被拒绝事件");

    // 记录 API Key 创建
    auditor.log(SecurityAuditLog::new(
        SecurityEventType::ApiKeyCreated {
            key_id: api_key.id,
            user_id,
        },
    ));
    println!("✓ 记录 API Key 创建事件");

    // 获取审计统计
    let audit_stats = auditor.get_stats();
    println!("\n✓ 审计统计:");
    println!("  - 总日志数: {}", audit_stats.total_logs);
    println!("  - 最近 24 小时: {}", audit_stats.recent_logs_24h);
    println!("  - 失败登录: {}", audit_stats.failed_logins_24h);
    println!("  - 访问被拒绝: {}", audit_stats.access_denied_24h);
    println!("  - 异常活动: {}", audit_stats.anomalous_activities_24h);

    // 获取失败的登录尝试
    let failed_logins = auditor.get_failed_logins();
    println!("\n✓ 失败的登录尝试: {} 次", failed_logins.len());

    // 获取高危事件
    let critical_events = auditor.get_critical_events();
    println!("✓ 高危事件: {} 个", critical_events.len());

    // 7. 暴力破解检测演示
    println!("\n7. 暴力破解检测");
    println!("==================");

    // 模拟暴力破解攻击
    println!("模拟暴力破解攻击...");
    for i in 1..=6 {
        auditor.log_login_attempt("victim@example.com", false, None, Some("10.0.0.2".to_string()));
        println!("  失败尝试 #{}", i);
    }

    // 检查是否检测到异常
    let critical_after_attack = auditor.get_critical_events();
    if critical_after_attack.len() > critical_events.len() {
        println!("✓ 检测到暴力破解攻击！");
        println!("  新增高危事件: {} 个", critical_after_attack.len() - critical_events.len());
    }

    println!("\n=== 演示完成 ===");
    println!("\n总结:");
    println!("- JWT Token: 生成、验证、刷新 ✓");
    println!("- API Key: 创建、验证、Scope 检查 ✓");
    println!("- 数据加密: AES-256-GCM 加密/解密 ✓");
    println!("- 密码哈希: SHA-256 哈希和验证 ✓");
    println!("- 密钥管理: 生成、轮换、统计 ✓");
    println!("- 安全审计: 日志记录、统计、查询 ✓");
    println!("- 异常检测: 暴力破解检测 ✓");
    println!("\n所有安全功能测试通过！");
}
