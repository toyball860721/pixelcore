use crate::models::{RestoreRecord, RestoreStatus};
use flate2::read::GzDecoder;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tar::Archive;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct RestoreManager {
    restores: Arc<Mutex<HashMap<Uuid, RestoreRecord>>>,
}

impl RestoreManager {
    pub fn new() -> Self {
        Self {
            restores: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 恢复备份
    pub fn restore_backup(
        &self,
        backup_id: Uuid,
        backup_path: &Path,
        target_path: &Path,
    ) -> io::Result<Uuid> {
        if !backup_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Backup file not found: {:?}", backup_path),
            ));
        }

        // 创建恢复记录
        let mut record = RestoreRecord::new(
            backup_id,
            backup_path.to_path_buf(),
            target_path.to_path_buf(),
        );

        let restore_id = record.id;

        // 执行恢复
        match self.perform_restore(backup_path, target_path) {
            Ok((file_count, restored_bytes)) => {
                record.complete(file_count, restored_bytes);
            }
            Err(e) => {
                record.fail(e.to_string());
                return Err(e);
            }
        }

        // 保存恢复记录
        let mut restores = self.restores.lock().unwrap();
        restores.insert(restore_id, record);

        Ok(restore_id)
    }

    /// 执行恢复操作
    fn perform_restore(&self, backup_path: &Path, target_path: &Path) -> io::Result<(usize, u64)> {
        // 创建目标目录
        fs::create_dir_all(target_path)?;

        // 打开备份文件
        let tar_gz = File::open(backup_path)?;
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);

        let mut file_count = 0usize;
        let mut restored_bytes = 0u64;

        // 解压文件
        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;
            let target_file = target_path.join(&*path);

            // 创建父目录
            if let Some(parent) = target_file.parent() {
                fs::create_dir_all(parent)?;
            }

            // 解压文件
            entry.unpack(&target_file)?;

            if let Ok(metadata) = fs::metadata(&target_file) {
                restored_bytes += metadata.len();
            }
            file_count += 1;
        }

        Ok((file_count, restored_bytes))
    }

    /// 获取恢复记录
    pub fn get_restore(&self, restore_id: Uuid) -> Option<RestoreRecord> {
        let restores = self.restores.lock().unwrap();
        restores.get(&restore_id).cloned()
    }

    /// 获取所有恢复记录
    pub fn list_restores(&self) -> Vec<RestoreRecord> {
        let restores = self.restores.lock().unwrap();
        let mut records: Vec<RestoreRecord> = restores.values().cloned().collect();
        records.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        records
    }

    /// 验证恢复
    pub fn verify_restore(&self, restore_id: Uuid) -> io::Result<bool> {
        let mut restores = self.restores.lock().unwrap();

        if let Some(record) = restores.get_mut(&restore_id) {
            // 检查目标路径是否存在
            if !record.target_path.exists() {
                return Ok(false);
            }

            // 验证文件数量
            let file_count = self.count_files(&record.target_path)?;
            if file_count != record.file_count {
                return Ok(false);
            }

            record.status = RestoreStatus::Verified;
            Ok(true)
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Restore not found: {}", restore_id),
            ))
        }
    }

    /// 计算目录中的文件数量
    fn count_files(&self, path: &Path) -> io::Result<usize> {
        let mut count = 0;

        if path.is_file() {
            return Ok(1);
        }

        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    count += 1;
                } else if path.is_dir() {
                    count += self.count_files(&path)?;
                }
            }
        }

        Ok(count)
    }
}

impl Default for RestoreManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backup::BackupManager;
    use tempfile::TempDir;

    #[test]
    fn test_restore_backup() {
        let temp_dir = TempDir::new().unwrap();
        let backup_root = temp_dir.path().join("backups");
        let source_dir = temp_dir.path().join("source");
        let restore_dir = temp_dir.path().join("restore");

        // 创建测试文件
        fs::create_dir_all(&source_dir).unwrap();
        fs::write(source_dir.join("test.txt"), b"Hello, World!").unwrap();
        fs::write(source_dir.join("test2.txt"), b"Test 2").unwrap();

        // 创建备份
        let backup_manager = BackupManager::new(backup_root).unwrap();
        let backup_id = backup_manager.create_full_backup(&source_dir, "test").unwrap();

        let backup_record = backup_manager.get_backup(backup_id).unwrap();

        // 恢复备份
        let restore_manager = RestoreManager::new();
        let restore_id = restore_manager
            .restore_backup(backup_id, &backup_record.backup_path, &restore_dir)
            .unwrap();

        let restore_record = restore_manager.get_restore(restore_id).unwrap();
        assert_eq!(restore_record.status, RestoreStatus::Completed);
        assert_eq!(restore_record.file_count, 2);

        // 验证恢复的文件
        assert!(restore_dir.join("test.txt").exists());
        assert!(restore_dir.join("test2.txt").exists());

        let content = fs::read_to_string(restore_dir.join("test.txt")).unwrap();
        assert_eq!(content, "Hello, World!");
    }

    #[test]
    fn test_verify_restore() {
        let temp_dir = TempDir::new().unwrap();
        let backup_root = temp_dir.path().join("backups");
        let source_dir = temp_dir.path().join("source");
        let restore_dir = temp_dir.path().join("restore");

        fs::create_dir_all(&source_dir).unwrap();
        fs::write(source_dir.join("test.txt"), b"Test").unwrap();

        let backup_manager = BackupManager::new(backup_root).unwrap();
        let backup_id = backup_manager.create_full_backup(&source_dir, "test").unwrap();

        let backup_record = backup_manager.get_backup(backup_id).unwrap();

        let restore_manager = RestoreManager::new();
        let restore_id = restore_manager
            .restore_backup(backup_id, &backup_record.backup_path, &restore_dir)
            .unwrap();

        let is_valid = restore_manager.verify_restore(restore_id).unwrap();
        assert!(is_valid);

        let record = restore_manager.get_restore(restore_id).unwrap();
        assert_eq!(record.status, RestoreStatus::Verified);
    }

    #[test]
    fn test_list_restores() {
        let temp_dir = TempDir::new().unwrap();
        let backup_root = temp_dir.path().join("backups");
        let source_dir = temp_dir.path().join("source");

        fs::create_dir_all(&source_dir).unwrap();
        fs::write(source_dir.join("test.txt"), b"Test").unwrap();

        let backup_manager = BackupManager::new(backup_root).unwrap();
        let backup_id = backup_manager.create_full_backup(&source_dir, "test").unwrap();

        let backup_record = backup_manager.get_backup(backup_id).unwrap();

        let restore_manager = RestoreManager::new();

        let restore_dir1 = temp_dir.path().join("restore1");
        let restore_dir2 = temp_dir.path().join("restore2");

        restore_manager
            .restore_backup(backup_id, &backup_record.backup_path, &restore_dir1)
            .unwrap();
        restore_manager
            .restore_backup(backup_id, &backup_record.backup_path, &restore_dir2)
            .unwrap();

        let restores = restore_manager.list_restores();
        assert_eq!(restores.len(), 2);
    }
}
