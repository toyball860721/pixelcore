use pixelcore_storage::{Storage, StorageError};
use serde_json::json;
use tempfile::TempDir;

#[test]
fn test_memory_storage() {
    let storage = Storage::new();

    // 测试 set 和 get
    storage.set("key1", json!("value1")).unwrap();
    assert_eq!(storage.get("key1").unwrap(), json!("value1"));

    // 测试 contains
    assert!(storage.contains("key1").unwrap());
    assert!(!storage.contains("key2").unwrap());

    // 测试 delete
    assert!(storage.delete("key1").unwrap());
    assert!(!storage.delete("key1").unwrap());

    // 测试 keys
    storage.set("a", json!(1)).unwrap();
    storage.set("b", json!(2)).unwrap();
    let keys = storage.keys().unwrap();
    assert_eq!(keys.len(), 2);
    assert!(keys.contains(&"a".to_string()));
    assert!(keys.contains(&"b".to_string()));
}

#[test]
fn test_sled_storage() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let storage = Storage::open(&db_path).unwrap();

    // 测试基本操作
    storage.set("key1", json!({"name": "test"})).unwrap();
    assert_eq!(storage.get("key1").unwrap(), json!({"name": "test"}));

    // 测试持久化：重新打开数据库
    drop(storage);
    let storage = Storage::open(&db_path).unwrap();
    assert_eq!(storage.get("key1").unwrap(), json!({"name": "test"}));
}

#[test]
fn test_encrypted_storage() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("encrypted.db");
    let key = "test-encryption-key-32-bytes!!";

    let storage = Storage::open_encrypted(&db_path, key).unwrap();

    // 测试加密存储的基本操作
    storage.set("secret", json!({"password": "12345"})).unwrap();
    assert_eq!(storage.get("secret").unwrap(), json!({"password": "12345"}));

    // 测试持久化和加密：重新打开数据库
    drop(storage);
    let storage = Storage::open_encrypted(&db_path, key).unwrap();
    assert_eq!(storage.get("secret").unwrap(), json!({"password": "12345"}));

    // 测试错误的密钥无法打开数据库
    let wrong_key = "wrong-key-32-bytes-long-here!!";
    let result = Storage::open_encrypted(&db_path, wrong_key);
    // SQLCipher 会在尝试读取时失败，而不是在打开时
    if let Ok(storage) = result {
        // 尝试读取应该失败
        assert!(storage.get("secret").is_err());
    }
}

#[test]
fn test_not_found_error() {
    let storage = Storage::new();
    match storage.get("nonexistent") {
        Err(StorageError::NotFound(key)) => {
            assert_eq!(key, "nonexistent");
        }
        _ => panic!("Expected NotFound error"),
    }
}

#[test]
fn test_complex_json_values() {
    let storage = Storage::new();

    let complex_value = json!({
        "users": [
            {"id": 1, "name": "Alice"},
            {"id": 2, "name": "Bob"}
        ],
        "metadata": {
            "version": "1.0",
            "timestamp": 1234567890
        }
    });

    storage.set("complex", complex_value.clone()).unwrap();
    assert_eq!(storage.get("complex").unwrap(), complex_value);
}

