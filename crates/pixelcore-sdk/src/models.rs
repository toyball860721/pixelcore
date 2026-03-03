use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 插件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: String,
    pub tags: Vec<String>,
    pub dependencies: Vec<PluginDependency>,
    pub created_at: DateTime<Utc>,
}

impl PluginMetadata {
    pub fn new(name: String, version: String, author: String, description: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            version,
            author,
            description,
            homepage: None,
            repository: None,
            license: "MIT".to_string(),
            tags: Vec::new(),
            dependencies: Vec::new(),
            created_at: Utc::now(),
        }
    }

    pub fn with_homepage(mut self, homepage: String) -> Self {
        self.homepage = Some(homepage);
        self
    }

    pub fn with_repository(mut self, repository: String) -> Self {
        self.repository = Some(repository);
        self
    }

    pub fn with_license(mut self, license: String) -> Self {
        self.license = license;
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn add_dependency(&mut self, dependency: PluginDependency) {
        self.dependencies.push(dependency);
    }
}

/// 插件依赖
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    pub name: String,
    pub version: String,
    pub optional: bool,
}

impl PluginDependency {
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            optional: false,
        }
    }

    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }
}

/// 插件状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginStatus {
    /// 未加载
    Unloaded,
    /// 已加载
    Loaded,
    /// 已启用
    Enabled,
    /// 已禁用
    Disabled,
    /// 错误
    Error,
}

/// 插件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub metadata: PluginMetadata,
    pub status: PluginStatus,
    pub loaded_at: Option<DateTime<Utc>>,
    pub enabled_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

impl PluginInfo {
    pub fn new(metadata: PluginMetadata) -> Self {
        Self {
            metadata,
            status: PluginStatus::Unloaded,
            loaded_at: None,
            enabled_at: None,
            error_message: None,
        }
    }

    pub fn load(&mut self) {
        self.status = PluginStatus::Loaded;
        self.loaded_at = Some(Utc::now());
    }

    pub fn enable(&mut self) {
        self.status = PluginStatus::Enabled;
        self.enabled_at = Some(Utc::now());
    }

    pub fn disable(&mut self) {
        self.status = PluginStatus::Disabled;
    }

    pub fn error(&mut self, message: String) {
        self.status = PluginStatus::Error;
        self.error_message = Some(message);
    }
}

/// SDK 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkConfig {
    pub api_endpoint: String,
    pub api_key: Option<String>,
    pub timeout_secs: u64,
    pub retry_count: u32,
    pub enable_logging: bool,
    pub log_level: String,
}

impl Default for SdkConfig {
    fn default() -> Self {
        Self {
            api_endpoint: "http://localhost:8080".to_string(),
            api_key: None,
            timeout_secs: 30,
            retry_count: 3,
            enable_logging: true,
            log_level: "info".to_string(),
        }
    }
}

impl SdkConfig {
    pub fn new(api_endpoint: String) -> Self {
        Self {
            api_endpoint,
            ..Default::default()
        }
    }

    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    pub fn with_retry_count(mut self, retry_count: u32) -> Self {
        self.retry_count = retry_count;
        self
    }
}

/// API 请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Option<serde_json::Value>,
}

impl ApiRequest {
    pub fn new(method: String, path: String) -> Self {
        Self {
            method,
            path,
            headers: HashMap::new(),
            body: None,
        }
    }

    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }

    pub fn with_body(mut self, body: serde_json::Value) -> Self {
        self.body = Some(body);
        self
    }
}

/// API 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<serde_json::Value>,
}

impl ApiResponse {
    pub fn new(status_code: u16) -> Self {
        Self {
            status_code,
            headers: HashMap::new(),
            body: None,
        }
    }

    pub fn is_success(&self) -> bool {
        self.status_code >= 200 && self.status_code < 300
    }

    pub fn is_error(&self) -> bool {
        self.status_code >= 400
    }
}

/// 插件事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginEvent {
    pub id: Uuid,
    pub plugin_id: Uuid,
    pub event_type: String,
    pub data: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

impl PluginEvent {
    pub fn new(plugin_id: Uuid, event_type: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            plugin_id,
            event_type,
            data: HashMap::new(),
            timestamp: Utc::now(),
        }
    }

    pub fn with_data(mut self, key: String, value: serde_json::Value) -> Self {
        self.data.insert(key, value);
        self
    }
}
