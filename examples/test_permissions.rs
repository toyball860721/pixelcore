use pixelcore_skills::{Permission, PermissionManager, PermissionCheck, FileOperation, StorageOperation};
use std::path::PathBuf;

fn main() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                                                              ║");
    println!("║           Skills 权限管理系统演示                             ║");
    println!("║                                                              ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    println!("═══ 第一部分：文件系统权限 ═══\n");

    let mut manager = PermissionManager::new();

    // 授予 /tmp 目录的读写权限
    manager.grant(Permission::FileSystem {
        path: PathBuf::from("/tmp"),
        read: true,
        write: true,
    });

    println!("✅ 已授予权限: /tmp 目录读写");

    // 测试允许的操作
    println!("\n【测试 1】读取 /tmp/test.txt");
    let allowed = manager.check(&PermissionCheck::FileSystem {
        path: PathBuf::from("/tmp/test.txt"),
        operation: FileOperation::Read,
    });
    println!("  结果: {}", if allowed { "✅ 允许" } else { "❌ 拒绝" });

    println!("\n【测试 2】写入 /tmp/data.json");
    let allowed = manager.check(&PermissionCheck::FileSystem {
        path: PathBuf::from("/tmp/data.json"),
        operation: FileOperation::Write,
    });
    println!("  结果: {}", if allowed { "✅ 允许" } else { "❌ 拒绝" });

    // 测试拒绝的操作
    println!("\n【测试 3】读取 /etc/passwd（不在授权目录）");
    let allowed = manager.check(&PermissionCheck::FileSystem {
        path: PathBuf::from("/etc/passwd"),
        operation: FileOperation::Read,
    });
    println!("  结果: {}", if allowed { "✅ 允许" } else { "❌ 拒绝" });

    println!("\n═══ 第二部分：网络权限 ═══\n");

    // 授予特定域名的网络权限
    manager.grant(Permission::Network {
        host: "*.example.com".to_string(),
        port: 443,
    });

    println!("✅ 已授予权限: *.example.com:443");

    println!("\n【测试 4】访问 api.example.com:443");
    let allowed = manager.check(&PermissionCheck::Network {
        host: "api.example.com".to_string(),
        port: 443,
    });
    println!("  结果: {}", if allowed { "✅ 允许" } else { "❌ 拒绝" });

    println!("\n【测试 5】访问 api.example.com:80（端口不匹配）");
    let allowed = manager.check(&PermissionCheck::Network {
        host: "api.example.com".to_string(),
        port: 80,
    });
    println!("  结果: {}", if allowed { "✅ 允许" } else { "❌ 拒绝" });

    println!("\n【测试 6】访问 evil.com:443（域名不匹配）");
    let allowed = manager.check(&PermissionCheck::Network {
        host: "evil.com".to_string(),
        port: 443,
    });
    println!("  结果: {}", if allowed { "✅ 允许" } else { "❌ 拒绝" });

    println!("\n═══ 第三部分：计算资源权限 ═══\n");

    // 授予计算资源权限
    manager.grant(Permission::Compute {
        max_time_ms: 5000,  // 最多 5 秒
        max_memory_mb: 100, // 最多 100 MB
    });

    println!("✅ 已授予权限: 最多 5 秒执行时间，100 MB 内存");

    println!("\n【测试 7】执行 2 秒任务，使用 50 MB 内存");
    let allowed = manager.check(&PermissionCheck::Compute {
        estimated_time_ms: 2000,
        estimated_memory_mb: 50,
    });
    println!("  结果: {}", if allowed { "✅ 允许" } else { "❌ 拒绝" });

    println!("\n【测试 8】执行 10 秒任务（超时）");
    let allowed = manager.check(&PermissionCheck::Compute {
        estimated_time_ms: 10000,
        estimated_memory_mb: 50,
    });
    println!("  结果: {}", if allowed { "✅ 允许" } else { "❌ 拒绝" });

    println!("\n═══ 第四部分：存储权限 ═══\n");

    // 授予存储权限
    manager.grant(Permission::Storage {
        namespace: "user_data".to_string(),
        read: true,
        write: true,
    });

    manager.grant(Permission::Storage {
        namespace: "cache".to_string(),
        read: true,
        write: false,  // 只读
    });

    println!("✅ 已授予权限: user_data（读写），cache（只读）");

    println!("\n【测试 9】读取 user_data");
    let allowed = manager.check(&PermissionCheck::Storage {
        namespace: "user_data".to_string(),
        operation: StorageOperation::Read,
    });
    println!("  结果: {}", if allowed { "✅ 允许" } else { "❌ 拒绝" });

    println!("\n【测试 10】写入 cache（只读命名空间）");
    let allowed = manager.check(&PermissionCheck::Storage {
        namespace: "cache".to_string(),
        operation: StorageOperation::Write,
    });
    println!("  结果: {}", if allowed { "✅ 允许" } else { "❌ 拒绝" });

    println!("\n═══ 第五部分：进程执行权限 ═══\n");

    // 授予进程执行权限
    manager.grant(Permission::Process {
        command: "python3".to_string(),
        args_pattern: Some("*.py".to_string()),
    });

    println!("✅ 已授予权限: python3 *.py");

    println!("\n【测试 11】执行 python3 script.py");
    let allowed = manager.check(&PermissionCheck::Process {
        command: "python3".to_string(),
        args: Some(vec!["script.py".to_string()]),
    });
    println!("  结果: {}", if allowed { "✅ 允许" } else { "❌ 拒绝" });

    println!("\n【测试 12】执行 python3 script.sh（扩展名不匹配）");
    let allowed = manager.check(&PermissionCheck::Process {
        command: "python3".to_string(),
        args: Some(vec!["script.sh".to_string()]),
    });
    println!("  结果: {}", if allowed { "✅ 允许" } else { "❌ 拒绝" });

    println!("\n═══ 权限总结 ═══\n");
    println!("已授予的权限数量: {}", manager.permissions().len());
    println!("\n权限列表:");
    for (i, perm) in manager.permissions().iter().enumerate() {
        println!("  {}. {:?}", i + 1, perm);
    }

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  演示完成！                                                   ║");
    println!("║                                                              ║");
    println!("║  权限管理系统可以有效控制 Skills 的访问权限                   ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");
}
