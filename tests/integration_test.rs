/// 集成测试：测试 Storage + Runtime 的集成
use pixelcore_storage::Storage;
use serde_json::json;
use tempfile::TempDir;

#[test]
fn test_storage_runtime_integration() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("integration.db");

    // 创建存储
    let storage = Storage::open(&db_path).unwrap();

    // 模拟 Agent 状态存储
    let agent_state = json!({
        "agent_id": "agent-001",
        "status": "running",
        "flow_level": 0.75,
        "tasks": [
            {"id": "task-1", "status": "completed"},
            {"id": "task-2", "status": "in_progress"}
        ]
    });

    storage.set("agent:agent-001", agent_state.clone()).unwrap();

    // 验证可以读取
    let retrieved = storage.get("agent:agent-001").unwrap();
    assert_eq!(retrieved, agent_state);

    // 模拟更新 Agent 状态
    let updated_state = json!({
        "agent_id": "agent-001",
        "status": "idle",
        "flow_level": 0.0,
        "tasks": []
    });

    storage.set("agent:agent-001", updated_state.clone()).unwrap();
    let retrieved = storage.get("agent:agent-001").unwrap();
    assert_eq!(retrieved, updated_state);
}

#[test]
fn test_encrypted_storage_integration() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("encrypted_integration.db");
    let key = "secure-key-for-testing-32byte";

    // 创建加密存储
    let storage = Storage::open_encrypted(&db_path, key).unwrap();

    // 存储敏感数据
    let sensitive_data = json!({
        "api_key": "sk-test-key-12345",
        "credentials": {
            "username": "admin",
            "password": "secret"
        }
    });

    storage.set("secrets:api", sensitive_data.clone()).unwrap();

    // 验证数据加密存储
    drop(storage);

    // 重新打开并验证
    let storage = Storage::open_encrypted(&db_path, key).unwrap();
    let retrieved = storage.get("secrets:api").unwrap();
    assert_eq!(retrieved, sensitive_data);
}
