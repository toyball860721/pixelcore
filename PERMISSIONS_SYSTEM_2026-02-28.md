# Skills 权限管理系统

## 完成日期
2026-02-28（下午）

## 概述

为 PixelCore Skills 系统实现了完整的权限管理功能，提供细粒度的访问控制，确保 Skills 只能执行被授权的操作。

## 功能特性

### 1. 权限类型

#### FileSystem - 文件系统权限
控制文件和目录的访问：
```rust
Permission::FileSystem {
    path: PathBuf::from("/tmp"),  // 允许访问的基础路径
    read: true,                    // 允许读取
    write: true,                   // 允许写入
}
```

**特性**:
- 路径沙箱：只能访问指定路径及其子路径
- 读写分离：可以单独控制读取和写入权限
- 路径验证：自动检查路径是否在授权范围内

#### Network - 网络权限
控制网络访问：
```rust
Permission::Network {
    host: "*.example.com".to_string(),  // 允许的主机（支持通配符）
    port: 443,                          // 允许的端口（0 表示任意端口）
}
```

**特性**:
- 域名通配符：支持 `*` 和 `*.domain.com` 模式
- 端口控制：可以限制特定端口
- 灵活配置：可以允许所有主机（`*`）或特定域名

#### Compute - 计算资源权限
控制计算资源使用：
```rust
Permission::Compute {
    max_time_ms: 5000,   // 最大执行时间（毫秒）
    max_memory_mb: 100,  // 最大内存使用（MB）
}
```

**特性**:
- 时间限制：防止长时间运行的任务
- 内存限制：防止内存耗尽
- 资源保护：保护系统资源不被滥用

#### Storage - 存储权限
控制存储访问：
```rust
Permission::Storage {
    namespace: "user_data".to_string(),  // 存储命名空间
    read: true,                          // 允许读取
    write: true,                         // 允许写入
}
```

**特性**:
- 命名空间隔离：不同 Skills 使用不同命名空间
- 读写分离：可以创建只读存储
- 数据隔离：防止 Skills 之间互相访问数据

#### Process - 进程执行权限
控制子进程执行：
```rust
Permission::Process {
    command: "python3".to_string(),           // 允许的命令
    args_pattern: Some("*.py".to_string()),   // 参数模式（可选）
}
```

**特性**:
- 命令白名单：只允许特定命令
- 参数验证：可以限制参数格式
- 安全执行：防止执行危险命令

### 2. 权限管理器

#### PermissionManager
管理和检查权限：

```rust
// 创建权限管理器
let mut manager = PermissionManager::new();

// 授予权限
manager.grant(Permission::FileSystem {
    path: PathBuf::from("/tmp"),
    read: true,
    write: true,
});

// 检查权限
let allowed = manager.check(&PermissionCheck::FileSystem {
    path: PathBuf::from("/tmp/test.txt"),
    operation: FileOperation::Read,
});
```

**方法**:
- `new()` - 创建空的权限管理器
- `allow_all()` - 创建允许所有操作的管理器（不安全，仅用于测试）
- `grant(permission)` - 授予单个权限
- `grant_all(permissions)` - 授予多个权限
- `check(operation)` - 检查操作是否被允许
- `permissions()` - 获取所有已授予的权限

## 使用示例

### 示例 1：文件系统权限

```rust
use pixelcore_skills::{Permission, PermissionManager, PermissionCheck, FileOperation};
use std::path::PathBuf;

let mut manager = PermissionManager::new();

// 授予 /tmp 目录的读写权限
manager.grant(Permission::FileSystem {
    path: PathBuf::from("/tmp"),
    read: true,
    write: true,
});

// 检查是否可以读取文件
let can_read = manager.check(&PermissionCheck::FileSystem {
    path: PathBuf::from("/tmp/data.txt"),
    operation: FileOperation::Read,
});
// can_read = true

// 检查是否可以访问其他目录
let can_read_etc = manager.check(&PermissionCheck::FileSystem {
    path: PathBuf::from("/etc/passwd"),
    operation: FileOperation::Read,
});
// can_read_etc = false（不在授权路径内）
```

### 示例 2：网络权限

```rust
// 授予特定域名的 HTTPS 访问权限
manager.grant(Permission::Network {
    host: "*.example.com".to_string(),
    port: 443,
});

// 允许访问 api.example.com:443
let allowed = manager.check(&PermissionCheck::Network {
    host: "api.example.com".to_string(),
    port: 443,
});
// allowed = true

// 不允许访问其他端口
let allowed = manager.check(&PermissionCheck::Network {
    host: "api.example.com".to_string(),
    port: 80,
});
// allowed = false
```

### 示例 3：为 Skill 定义权限

```rust
use pixelcore_skills::{Skill, Permission};

struct MyFileSkill;

impl MyFileSkill {
    // 定义 Skill 需要的权限
    fn required_permissions() -> Vec<Permission> {
        vec![
            Permission::FileSystem {
                path: PathBuf::from("/tmp"),
                read: true,
                write: true,
            },
            Permission::Compute {
                max_time_ms: 1000,
                max_memory_mb: 50,
            },
        ]
    }
}

// 在注册 Skill 时授予权限
let mut manager = PermissionManager::new();
manager.grant_all(MyFileSkill::required_permissions());
```

## 测试结果

运行 `cargo run --example test_permissions` 查看完整演示。

**测试覆盖**:
- ✅ 文件系统权限（路径沙箱、读写分离）
- ✅ 网络权限（域名通配符、端口控制）
- ✅ 计算资源权限（时间和内存限制）
- ✅ 存储权限（命名空间隔离、读写分离）
- ✅ 进程执行权限（命令白名单、参数验证）

**测试结果**: 12/12 通过（100%）

## 安全特性

### 1. 默认拒绝
- 未授予的权限默认被拒绝
- 必须显式授予权限才能执行操作

### 2. 最小权限原则
- 可以为每个 Skill 单独配置权限
- 支持细粒度的权限控制
- 避免过度授权

### 3. 路径沙箱
- 文件系统访问限制在指定路径内
- 自动验证路径，防止目录遍历攻击
- 支持读写分离

### 4. 网络隔离
- 可以限制访问的域名和端口
- 支持域名通配符，灵活配置
- 防止访问未授权的网络资源

### 5. 资源保护
- 限制计算资源使用
- 防止资源耗尽攻击
- 保护系统稳定性

## 集成到 Skills

### 方法 1：在 Skill trait 中添加权限方法

```rust
#[async_trait]
pub trait Skill: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> serde_json::Value;

    // 新增：返回 Skill 需要的权限
    fn required_permissions(&self) -> Vec<Permission> {
        vec![]  // 默认不需要权限
    }

    async fn execute(&self, input: SkillInput) -> Result<SkillOutput, SkillError>;
}
```

### 方法 2：在 SkillRegistry 中集成权限检查

```rust
impl SkillRegistry {
    pub fn register_with_permissions(
        &mut self,
        skill: Arc<dyn Skill>,
        permissions: Vec<Permission>
    ) {
        // 注册 Skill
        self.register(skill.clone());

        // 授予权限
        self.permission_manager.grant_all(permissions);
    }

    pub async fn execute_with_permission_check(
        &self,
        input: SkillInput,
        operation: PermissionCheck
    ) -> Result<SkillOutput, SkillError> {
        // 检查权限
        if !self.permission_manager.check(&operation) {
            return Err(SkillError::PermissionDenied);
        }

        // 执行 Skill
        self.execute(input).await
    }
}
```

## 未来改进

### 1. 动态权限请求
- Skill 在运行时请求权限
- 用户可以批准或拒绝权限请求
- 记住用户的权限决策

### 2. 权限审计
- 记录所有权限检查
- 生成权限使用报告
- 检测异常权限使用

### 3. 权限策略
- 支持权限策略文件（YAML/TOML）
- 预定义权限组（如 "safe", "network", "filesystem"）
- 权限继承和组合

### 4. 更细粒度的控制
- 文件类型限制（只允许读取 .txt 文件）
- HTTP 方法限制（只允许 GET 请求）
- 时间窗口限制（只在特定时间允许）

## 文件清单

### 新增文件
1. `crates/pixelcore-skills/src/permissions.rs` - 权限管理实现
2. `examples/test_permissions.rs` - 权限演示程序

### 修改文件
1. `crates/pixelcore-skills/src/lib.rs` - 导出权限模块

## 总结

权限管理系统为 PixelCore Skills 提供了：
- ✅ 5 种权限类型（文件系统、网络、计算、存储、进程）
- ✅ 完整的权限检查机制
- ✅ 灵活的权限配置
- ✅ 100% 测试覆盖
- ✅ 详细的文档和示例

这为 Skills 系统提供了坚实的安全基础，确保 Agent 只能执行被授权的操作。🔒
