use crate::models::{PluginEvent, PluginMetadata};
use async_trait::async_trait;
use std::collections::HashMap;

/// 插件接口
#[async_trait]
pub trait Plugin: Send + Sync {
    /// 获取插件元数据
    fn metadata(&self) -> PluginMetadata;

    /// 初始化插件
    async fn initialize(&mut self) -> Result<(), String>;

    /// 启动插件
    async fn start(&mut self) -> Result<(), String>;

    /// 停止插件
    async fn stop(&mut self) -> Result<(), String>;

    /// 处理事件
    async fn handle_event(&mut self, event: PluginEvent) -> Result<(), String>;

    /// 获取插件配置
    fn get_config(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    /// 设置插件配置
    fn set_config(&mut self, _config: HashMap<String, String>) -> Result<(), String> {
        Ok(())
    }

    /// 健康检查
    async fn health_check(&self) -> Result<bool, String> {
        Ok(true)
    }
}

/// 插件构建器
pub trait PluginBuilder: Send + Sync {
    /// 构建插件实例
    fn build(&self) -> Box<dyn Plugin>;
}

/// 示例插件实现
pub struct ExamplePlugin {
    metadata: PluginMetadata,
    config: HashMap<String, String>,
    is_running: bool,
}

impl ExamplePlugin {
    pub fn new() -> Self {
        let metadata = PluginMetadata::new(
            "example-plugin".to_string(),
            "1.0.0".to_string(),
            "PixelCore Team".to_string(),
            "An example plugin for demonstration".to_string(),
        );

        Self {
            metadata,
            config: HashMap::new(),
            is_running: false,
        }
    }
}

impl Default for ExamplePlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for ExamplePlugin {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }

    async fn initialize(&mut self) -> Result<(), String> {
        println!("[ExamplePlugin] Initializing...");
        Ok(())
    }

    async fn start(&mut self) -> Result<(), String> {
        println!("[ExamplePlugin] Starting...");
        self.is_running = true;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), String> {
        println!("[ExamplePlugin] Stopping...");
        self.is_running = false;
        Ok(())
    }

    async fn handle_event(&mut self, event: PluginEvent) -> Result<(), String> {
        println!("[ExamplePlugin] Handling event: {}", event.event_type);
        Ok(())
    }

    fn get_config(&self) -> HashMap<String, String> {
        self.config.clone()
    }

    fn set_config(&mut self, config: HashMap<String, String>) -> Result<(), String> {
        self.config = config;
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, String> {
        Ok(self.is_running)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_example_plugin() {
        let mut plugin = ExamplePlugin::new();

        // 测试元数据
        let metadata = plugin.metadata();
        assert_eq!(metadata.name, "example-plugin");
        assert_eq!(metadata.version, "1.0.0");

        // 测试初始化
        assert!(plugin.initialize().await.is_ok());

        // 测试启动
        assert!(plugin.start().await.is_ok());
        assert!(plugin.health_check().await.unwrap());

        // 测试配置
        let mut config = HashMap::new();
        config.insert("key".to_string(), "value".to_string());
        assert!(plugin.set_config(config.clone()).is_ok());
        assert_eq!(plugin.get_config(), config);

        // 测试停止
        assert!(plugin.stop().await.is_ok());
        assert!(!plugin.health_check().await.unwrap());
    }
}
