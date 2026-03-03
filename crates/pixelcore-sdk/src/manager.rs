use crate::models::{PluginEvent, PluginInfo, PluginMetadata, PluginStatus};
use crate::plugin::Plugin;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Clone)]
pub struct PluginManager {
    plugins: Arc<Mutex<HashMap<Uuid, Box<dyn Plugin>>>>,
    plugin_info: Arc<Mutex<HashMap<Uuid, PluginInfo>>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(Mutex::new(HashMap::new())),
            plugin_info: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 注册插件
    pub fn register_plugin(&self, plugin: Box<dyn Plugin>) -> Result<Uuid, String> {
        let metadata = plugin.metadata();
        let plugin_id = metadata.id;

        let mut plugins = self.plugins.lock().unwrap();
        let mut plugin_info = self.plugin_info.lock().unwrap();

        // 检查是否已存在
        if plugins.contains_key(&plugin_id) {
            return Err(format!("Plugin already registered: {}", plugin_id));
        }

        // 注册插件
        plugins.insert(plugin_id, plugin);

        // 创建插件信息
        let info = PluginInfo::new(metadata);
        plugin_info.insert(plugin_id, info);

        Ok(plugin_id)
    }

    /// 加载插件
    pub async fn load_plugin(&self, plugin_id: Uuid) -> Result<(), String> {
        let mut plugins = self.plugins.lock().unwrap();
        let mut plugin_info = self.plugin_info.lock().unwrap();

        let plugin = plugins.get_mut(&plugin_id)
            .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;

        let info = plugin_info.get_mut(&plugin_id)
            .ok_or_else(|| format!("Plugin info not found: {}", plugin_id))?;

        // 初始化插件
        match plugin.initialize().await {
            Ok(_) => {
                info.load();
                Ok(())
            }
            Err(e) => {
                info.error(e.clone());
                Err(e)
            }
        }
    }

    /// 启用插件
    pub async fn enable_plugin(&self, plugin_id: Uuid) -> Result<(), String> {
        let mut plugins = self.plugins.lock().unwrap();
        let mut plugin_info = self.plugin_info.lock().unwrap();

        let plugin = plugins.get_mut(&plugin_id)
            .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;

        let info = plugin_info.get_mut(&plugin_id)
            .ok_or_else(|| format!("Plugin info not found: {}", plugin_id))?;

        // 启动插件
        match plugin.start().await {
            Ok(_) => {
                info.enable();
                Ok(())
            }
            Err(e) => {
                info.error(e.clone());
                Err(e)
            }
        }
    }

    /// 禁用插件
    pub async fn disable_plugin(&self, plugin_id: Uuid) -> Result<(), String> {
        let mut plugins = self.plugins.lock().unwrap();
        let mut plugin_info = self.plugin_info.lock().unwrap();

        let plugin = plugins.get_mut(&plugin_id)
            .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;

        let info = plugin_info.get_mut(&plugin_id)
            .ok_or_else(|| format!("Plugin info not found: {}", plugin_id))?;

        // 停止插件
        match plugin.stop().await {
            Ok(_) => {
                info.disable();
                Ok(())
            }
            Err(e) => {
                info.error(e.clone());
                Err(e)
            }
        }
    }

    /// 卸载插件
    pub fn unregister_plugin(&self, plugin_id: Uuid) -> Result<(), String> {
        let mut plugins = self.plugins.lock().unwrap();
        let mut plugin_info = self.plugin_info.lock().unwrap();

        plugins.remove(&plugin_id)
            .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;

        plugin_info.remove(&plugin_id);

        Ok(())
    }

    /// 发送事件到插件
    pub async fn send_event(&self, plugin_id: Uuid, event: PluginEvent) -> Result<(), String> {
        let mut plugins = self.plugins.lock().unwrap();

        let plugin = plugins.get_mut(&plugin_id)
            .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;

        plugin.handle_event(event).await
    }

    /// 广播事件到所有启用的插件
    pub async fn broadcast_event(&self, event: PluginEvent) -> Result<(), String> {
        let plugin_info = self.plugin_info.lock().unwrap();
        let enabled_plugins: Vec<Uuid> = plugin_info
            .iter()
            .filter(|(_, info)| info.status == PluginStatus::Enabled)
            .map(|(id, _)| *id)
            .collect();

        drop(plugin_info);

        for plugin_id in enabled_plugins {
            let _ = self.send_event(plugin_id, event.clone()).await;
        }

        Ok(())
    }

    /// 获取插件信息
    pub fn get_plugin_info(&self, plugin_id: Uuid) -> Option<PluginInfo> {
        let plugin_info = self.plugin_info.lock().unwrap();
        plugin_info.get(&plugin_id).cloned()
    }

    /// 列出所有插件
    pub fn list_plugins(&self) -> Vec<PluginInfo> {
        let plugin_info = self.plugin_info.lock().unwrap();
        plugin_info.values().cloned().collect()
    }

    /// 获取启用的插件数量
    pub fn count_enabled_plugins(&self) -> usize {
        let plugin_info = self.plugin_info.lock().unwrap();
        plugin_info.values()
            .filter(|info| info.status == PluginStatus::Enabled)
            .count()
    }

    /// 健康检查所有插件
    pub async fn health_check_all(&self) -> HashMap<Uuid, bool> {
        let mut plugins = self.plugins.lock().unwrap();
        let mut results = HashMap::new();

        for (id, plugin) in plugins.iter_mut() {
            let is_healthy = plugin.health_check().await.unwrap_or(false);
            results.insert(*id, is_healthy);
        }

        results
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::ExamplePlugin;

    #[tokio::test]
    async fn test_register_plugin() {
        let manager = PluginManager::new();
        let plugin = Box::new(ExamplePlugin::new());

        let plugin_id = manager.register_plugin(plugin).unwrap();
        assert!(manager.get_plugin_info(plugin_id).is_some());
    }

    #[tokio::test]
    async fn test_load_and_enable_plugin() {
        let manager = PluginManager::new();
        let plugin = Box::new(ExamplePlugin::new());

        let plugin_id = manager.register_plugin(plugin).unwrap();

        // 加载插件
        assert!(manager.load_plugin(plugin_id).await.is_ok());
        let info = manager.get_plugin_info(plugin_id).unwrap();
        assert_eq!(info.status, PluginStatus::Loaded);

        // 启用插件
        assert!(manager.enable_plugin(plugin_id).await.is_ok());
        let info = manager.get_plugin_info(plugin_id).unwrap();
        assert_eq!(info.status, PluginStatus::Enabled);
    }

    #[tokio::test]
    async fn test_disable_plugin() {
        let manager = PluginManager::new();
        let plugin = Box::new(ExamplePlugin::new());

        let plugin_id = manager.register_plugin(plugin).unwrap();
        manager.load_plugin(plugin_id).await.unwrap();
        manager.enable_plugin(plugin_id).await.unwrap();

        // 禁用插件
        assert!(manager.disable_plugin(plugin_id).await.is_ok());
        let info = manager.get_plugin_info(plugin_id).unwrap();
        assert_eq!(info.status, PluginStatus::Disabled);
    }

    #[tokio::test]
    async fn test_unregister_plugin() {
        let manager = PluginManager::new();
        let plugin = Box::new(ExamplePlugin::new());

        let plugin_id = manager.register_plugin(plugin).unwrap();
        assert!(manager.get_plugin_info(plugin_id).is_some());

        manager.unregister_plugin(plugin_id).unwrap();
        assert!(manager.get_plugin_info(plugin_id).is_none());
    }

    #[tokio::test]
    async fn test_send_event() {
        let manager = PluginManager::new();
        let plugin = Box::new(ExamplePlugin::new());

        let plugin_id = manager.register_plugin(plugin).unwrap();
        manager.load_plugin(plugin_id).await.unwrap();
        manager.enable_plugin(plugin_id).await.unwrap();

        let event = PluginEvent::new(plugin_id, "test_event".to_string());
        assert!(manager.send_event(plugin_id, event).await.is_ok());
    }

    #[tokio::test]
    async fn test_list_plugins() {
        let manager = PluginManager::new();

        let plugin1 = Box::new(ExamplePlugin::new());
        let plugin2 = Box::new(ExamplePlugin::new());

        manager.register_plugin(plugin1).unwrap();
        manager.register_plugin(plugin2).unwrap();

        let plugins = manager.list_plugins();
        assert_eq!(plugins.len(), 2);
    }

    #[tokio::test]
    async fn test_count_enabled_plugins() {
        let manager = PluginManager::new();

        let plugin1 = Box::new(ExamplePlugin::new());
        let plugin2 = Box::new(ExamplePlugin::new());

        let id1 = manager.register_plugin(plugin1).unwrap();
        let id2 = manager.register_plugin(plugin2).unwrap();

        manager.load_plugin(id1).await.unwrap();
        manager.enable_plugin(id1).await.unwrap();

        manager.load_plugin(id2).await.unwrap();
        manager.enable_plugin(id2).await.unwrap();

        assert_eq!(manager.count_enabled_plugins(), 2);

        manager.disable_plugin(id1).await.unwrap();
        assert_eq!(manager.count_enabled_plugins(), 1);
    }
}
