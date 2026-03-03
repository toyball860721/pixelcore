use pixelcore_sdk::{
    ExamplePlugin, Plugin, PluginEvent, PluginManager, PluginMetadata,
    SdkClientBuilder, PluginDependency,
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PixelCore SDK and Plugin System Demo ===\n");

    // 1. SDK 客户端演示
    println!("1. SDK Client Demo");
    let client = SdkClientBuilder::new("http://localhost:8080".to_string())
        .with_api_key("demo_api_key".to_string())
        .with_timeout(30)
        .with_retry_count(3)
        .build();

    println!("  Created SDK client");
    println!("  API Endpoint: {}", client.config().api_endpoint);
    println!("  API Key: {:?}", client.config().api_key);
    println!("  Timeout: {} seconds", client.config().timeout_secs);
    println!();

    // 发送 API 请求
    println!("  Sending GET request...");
    let response = client.get("/api/status").await?;
    println!("  Response status: {}", response.status_code);
    println!("  Response body: {:?}", response.body);
    println!();

    // 2. 插件系统演示
    println!("2. Plugin System Demo");
    let manager = PluginManager::new();
    println!("  Created plugin manager");
    println!();

    // 3. 注册插件
    println!("3. Registering Plugins");
    let plugin1 = Box::new(ExamplePlugin::new());
    let plugin1_id = manager.register_plugin(plugin1)?;
    println!("  Registered plugin 1: {}", plugin1_id);

    let plugin2 = Box::new(ExamplePlugin::new());
    let plugin2_id = manager.register_plugin(plugin2)?;
    println!("  Registered plugin 2: {}", plugin2_id);

    println!("  Total plugins: {}", manager.list_plugins().len());
    println!();

    // 4. 加载和启用插件
    println!("4. Loading and Enabling Plugins");
    manager.load_plugin(plugin1_id).await?;
    println!("  Loaded plugin 1");

    manager.enable_plugin(plugin1_id).await?;
    println!("  Enabled plugin 1");

    manager.load_plugin(plugin2_id).await?;
    println!("  Loaded plugin 2");

    manager.enable_plugin(plugin2_id).await?;
    println!("  Enabled plugin 2");

    println!("  Enabled plugins: {}", manager.count_enabled_plugins());
    println!();

    // 5. 查看插件信息
    println!("5. Plugin Information");
    let plugins = manager.list_plugins();
    for (i, info) in plugins.iter().enumerate() {
        println!("  Plugin {}:", i + 1);
        println!("    ID: {}", info.metadata.id);
        println!("    Name: {}", info.metadata.name);
        println!("    Version: {}", info.metadata.version);
        println!("    Author: {}", info.metadata.author);
        println!("    Status: {:?}", info.status);
        if let Some(loaded_at) = info.loaded_at {
            println!("    Loaded at: {}", loaded_at.format("%Y-%m-%d %H:%M:%S"));
        }
        if let Some(enabled_at) = info.enabled_at {
            println!("    Enabled at: {}", enabled_at.format("%Y-%m-%d %H:%M:%S"));
        }
    }
    println!();

    // 6. 发送事件到插件
    println!("6. Sending Events to Plugins");
    let event = PluginEvent::new(plugin1_id, "user_login".to_string())
        .with_data("user_id".to_string(), serde_json::json!("user_123"))
        .with_data("timestamp".to_string(), serde_json::json!(chrono::Utc::now().to_rfc3339()));

    manager.send_event(plugin1_id, event.clone()).await?;
    println!("  Sent event to plugin 1");
    println!();

    // 7. 广播事件
    println!("7. Broadcasting Events");
    let broadcast_event = PluginEvent::new(plugin1_id, "system_update".to_string())
        .with_data("version".to_string(), serde_json::json!("2.0.0"));

    manager.broadcast_event(broadcast_event).await?;
    println!("  Broadcasted event to all enabled plugins");
    println!();

    // 8. 健康检查
    println!("8. Health Check");
    let health_results = manager.health_check_all().await;
    for (plugin_id, is_healthy) in health_results {
        println!("  Plugin {}: {}", plugin_id, if is_healthy { "✓ Healthy" } else { "✗ Unhealthy" });
    }
    println!();

    // 9. 禁用插件
    println!("9. Disabling Plugin");
    manager.disable_plugin(plugin1_id).await?;
    println!("  Disabled plugin 1");
    println!("  Enabled plugins: {}", manager.count_enabled_plugins());
    println!();

    // 10. 卸载插件
    println!("10. Unregistering Plugin");
    manager.unregister_plugin(plugin1_id)?;
    println!("  Unregistered plugin 1");
    println!("  Total plugins: {}", manager.list_plugins().len());
    println!();

    // 11. 插件元数据演示
    println!("11. Plugin Metadata Demo");
    let mut metadata = PluginMetadata::new(
        "advanced-plugin".to_string(),
        "2.0.0".to_string(),
        "PixelCore Team".to_string(),
        "An advanced plugin with dependencies".to_string(),
    );

    metadata = metadata
        .with_homepage("https://pixelcore.dev/plugins/advanced".to_string())
        .with_repository("https://github.com/pixelcore/advanced-plugin".to_string())
        .with_license("Apache-2.0".to_string())
        .with_tags(vec!["advanced".to_string(), "analytics".to_string()]);

    metadata.add_dependency(PluginDependency::new(
        "base-plugin".to_string(),
        "1.0.0".to_string(),
    ));

    metadata.add_dependency(PluginDependency::new(
        "utils-plugin".to_string(),
        "0.5.0".to_string(),
    ).optional());

    println!("  Plugin: {}", metadata.name);
    println!("  Version: {}", metadata.version);
    println!("  Author: {}", metadata.author);
    println!("  License: {}", metadata.license);
    println!("  Homepage: {:?}", metadata.homepage);
    println!("  Repository: {:?}", metadata.repository);
    println!("  Tags: {:?}", metadata.tags);
    println!("  Dependencies:");
    for dep in &metadata.dependencies {
        println!("    - {} v{} {}", dep.name, dep.version, if dep.optional { "(optional)" } else { "" });
    }
    println!();

    println!("=== Demo Complete ===");
    Ok(())
}
