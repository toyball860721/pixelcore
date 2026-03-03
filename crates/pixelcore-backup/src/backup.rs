use crate::models::{BackupRecord, BackupStats, BackupStatus, BackupType};
use flate2::write::GzEncoder;
use flate2::Compression;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tar::Builder;
use uuid::Uuid;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct BackupManager {
    backups: Arc<Mutex<HashMap<Uuid, BackupRecord>>>,
    backup_root: PathBuf,
}

impl BackupManager {
    pub fn new(backup_root: PathBuf) -> io::Result<Self> {
        // 创建备份根目录
        fs::create_dir_all(&backup_root)?;

        Ok(Self {
            backups: Arc::new(Mutex::new(HashMap::new())),
            backup_root,
        })
    }

    /// 创建全量备份
    pub fn create_full_backup(
        &self,
        source_path: &Path,
        name: &str,
    ) -> io::Result<Uuid> {
        self.create_backup(source_path, name, BackupType::Full)
    }

    /// 创建增量备份
    pub fn create_incremental_backup(
        &self,
        source_path: &Path,
        name: &str,
    ) -> io::Result<Uuid> {
        self.create_backup(source_path, name, BackupType::Incremental)
    }

    /// 创建备份
    fn create_backup(
        &self,
        source_path: &Path,
        name: &str,
        backup_type: BackupType,
    ) -> io::Result<Uuid> {
        if !source_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Source path not found: {:?}", source_path),
            ));
        }

        // 生成备份文件名
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_filename = format!("{}_{}.tar.gz", name, timestamp);
        let backup_path = self.backup_root.join(&backup_filename);

        // 创建备份记录
        let mut record = BackupRecord::new(
            backup_type,
            source_path.to_path_buf(),
            backup_path.clone(),
        );

        let backup_id = record.id;

        // 执行备份
        match self.perform_backup(source_path, &backup_path) {
            Ok((size, file_count)) => {
                record.complete(size, file_count);

                // 计算压缩后的大小
                if let Ok(metadata) = fs::metadata(&backup_path) {
                    record.compressed_size_bytes = Some(metadata.len());
                }
            }
            Err(e) => {
                record.fail(e.to_string());
                return Err(e);
            }
        }

        // 保存备份记录
        let mut backups = self.backups.lock().unwrap();
        backups.insert(backup_id, record);

        Ok(backup_id)
    }

    /// 执行备份操作
    fn perform_backup(&self, source: &Path, target: &Path) -> io::Result<(u64, usize)> {
        let tar_gz = File::create(target)?;
        let enc = GzEncoder::new(tar_gz, Compression::default());
        let mut tar = Builder::new(enc);

        let mut total_size = 0u64;
        let mut file_count = 0usize;

        if source.is_file() {
            // 备份单个文件
            tar.append_path_with_name(source, source.file_name().unwrap())?;
            if let Ok(metadata) = fs::metadata(source) {
                total_size = metadata.len();
            }
            file_count = 1;
        } else if source.is_dir() {
            // 备份目录
            for entry in WalkDir::new(source).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() {
                    let relative_path = path.strip_prefix(source).unwrap();
                    tar.append_path_with_name(path, relative_path)?;

                    if let Ok(metadata) = fs::metadata(path) {
                        total_size += metadata.len();
                    }
                    file_count += 1;
                }
            }
        }

        tar.finish()?;

        Ok((total_size, file_count))
    }

    /// 获取备份记录
    pub fn get_backup(&self, backup_id: Uuid) -> Option<BackupRecord> {
        let backups = self.backups.lock().unwrap();
        backups.get(&backup_id).cloned()
    }

    /// 获取所有备份
    pub fn list_backups(&self) -> Vec<BackupRecord> {
        let backups = self.backups.lock().unwrap();
        let mut records: Vec<BackupRecord> = backups.values().cloned().collect();
        records.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        records
    }

    /// 删除备份
    pub fn delete_backup(&self, backup_id: Uuid) -> io::Result<()> {
        let mut backups = self.backups.lock().unwrap();

        if let Some(record) = backups.remove(&backup_id) {
            // 删除备份文件
            if record.backup_path.exists() {
                fs::remove_file(&record.backup_path)?;
            }
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Backup not found: {}", backup_id),
            ))
        }
    }

    /// 验证备份
    pub fn verify_backup(&self, backup_id: Uuid) -> io::Result<bool> {
        let mut backups = self.backups.lock().unwrap();

        if let Some(record) = backups.get_mut(&backup_id) {
            // 检查备份文件是否存在
            if !record.backup_path.exists() {
                return Ok(false);
            }

            // 计算校验和
            let checksum = self.calculate_checksum(&record.backup_path)?;
            record.verify(checksum);

            Ok(true)
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Backup not found: {}", backup_id),
            ))
        }
    }

    /// 计算文件校验和 (简单的 CRC)
    fn calculate_checksum(&self, path: &Path) -> io::Result<String> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        // 简单的校验和计算
        let checksum: u64 = buffer.iter().map(|&b| b as u64).sum();
        Ok(format!("{:x}", checksum))
    }

    /// 获取备份统计
    pub fn get_stats(&self) -> BackupStats {
        let backups = self.backups.lock().unwrap();
        let mut stats = BackupStats::new();

        stats.total_backups = backups.len();

        for record in backups.values() {
            stats.total_size_bytes += record.size_bytes;
            if let Some(compressed) = record.compressed_size_bytes {
                stats.total_compressed_size_bytes += compressed;
            }

            match record.status {
                BackupStatus::Completed | BackupStatus::Verified => {
                    stats.successful_backups += 1;
                }
                BackupStatus::Failed => {
                    stats.failed_backups += 1;
                }
                _ => {}
            }

            if let Some(duration) = record.duration_ms {
                stats.average_duration_ms += duration;
            }

            // 更新时间范围
            if let Some(oldest) = stats.oldest_backup {
                stats.oldest_backup = Some(oldest.min(record.created_at));
            } else {
                stats.oldest_backup = Some(record.created_at);
            }

            if let Some(newest) = stats.newest_backup {
                stats.newest_backup = Some(newest.max(record.created_at));
            } else {
                stats.newest_backup = Some(record.created_at);
            }
        }

        if stats.successful_backups > 0 {
            stats.average_duration_ms /= stats.successful_backups as u64;
        }

        stats
    }

    /// 清理旧备份
    pub fn cleanup_old_backups(&self, keep_count: usize) -> io::Result<usize> {
        let mut backups = self.backups.lock().unwrap();
        let mut records: Vec<BackupRecord> = backups.values().cloned().collect();

        // 按时间排序
        records.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let mut deleted_count = 0;

        // 删除超过保留数量的备份
        for record in records.iter().skip(keep_count) {
            if record.backup_path.exists() {
                fs::remove_file(&record.backup_path)?;
            }
            backups.remove(&record.id);
            deleted_count += 1;
        }

        Ok(deleted_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_backup_manager() {
        let temp_dir = TempDir::new().unwrap();
        let backup_root = temp_dir.path().join("backups");

        let manager = BackupManager::new(backup_root.clone()).unwrap();
        assert!(backup_root.exists());
    }

    #[test]
    fn test_create_full_backup() {
        let temp_dir = TempDir::new().unwrap();
        let backup_root = temp_dir.path().join("backups");
        let source_dir = temp_dir.path().join("source");

        // 创建测试文件
        fs::create_dir_all(&source_dir).unwrap();
        fs::write(source_dir.join("test.txt"), b"Hello, World!").unwrap();

        let manager = BackupManager::new(backup_root).unwrap();
        let backup_id = manager.create_full_backup(&source_dir, "test").unwrap();

        let record = manager.get_backup(backup_id).unwrap();
        assert_eq!(record.backup_type, BackupType::Full);
        assert_eq!(record.status, BackupStatus::Completed);
        assert!(record.backup_path.exists());
        assert_eq!(record.file_count, 1);
    }

    #[test]
    fn test_list_backups() {
        let temp_dir = TempDir::new().unwrap();
        let backup_root = temp_dir.path().join("backups");
        let source_dir = temp_dir.path().join("source");

        fs::create_dir_all(&source_dir).unwrap();
        fs::write(source_dir.join("test.txt"), b"Test").unwrap();

        let manager = BackupManager::new(backup_root).unwrap();

        manager.create_full_backup(&source_dir, "backup1").unwrap();
        manager.create_full_backup(&source_dir, "backup2").unwrap();

        let backups = manager.list_backups();
        assert_eq!(backups.len(), 2);
    }

    #[test]
    fn test_delete_backup() {
        let temp_dir = TempDir::new().unwrap();
        let backup_root = temp_dir.path().join("backups");
        let source_dir = temp_dir.path().join("source");

        fs::create_dir_all(&source_dir).unwrap();
        fs::write(source_dir.join("test.txt"), b"Test").unwrap();

        let manager = BackupManager::new(backup_root).unwrap();
        let backup_id = manager.create_full_backup(&source_dir, "test").unwrap();

        assert!(manager.get_backup(backup_id).is_some());

        manager.delete_backup(backup_id).unwrap();
        assert!(manager.get_backup(backup_id).is_none());
    }

    #[test]
    fn test_verify_backup() {
        let temp_dir = TempDir::new().unwrap();
        let backup_root = temp_dir.path().join("backups");
        let source_dir = temp_dir.path().join("source");

        fs::create_dir_all(&source_dir).unwrap();
        fs::write(source_dir.join("test.txt"), b"Test").unwrap();

        let manager = BackupManager::new(backup_root).unwrap();
        let backup_id = manager.create_full_backup(&source_dir, "test").unwrap();

        let is_valid = manager.verify_backup(backup_id).unwrap();
        assert!(is_valid);

        let record = manager.get_backup(backup_id).unwrap();
        assert_eq!(record.status, BackupStatus::Verified);
        assert!(record.checksum.is_some());
    }

    #[test]
    fn test_get_stats() {
        let temp_dir = TempDir::new().unwrap();
        let backup_root = temp_dir.path().join("backups");
        let source_dir = temp_dir.path().join("source");

        fs::create_dir_all(&source_dir).unwrap();
        fs::write(source_dir.join("test.txt"), b"Test").unwrap();

        let manager = BackupManager::new(backup_root).unwrap();

        manager.create_full_backup(&source_dir, "backup1").unwrap();
        manager.create_full_backup(&source_dir, "backup2").unwrap();

        let stats = manager.get_stats();
        assert_eq!(stats.total_backups, 2);
        assert_eq!(stats.successful_backups, 2);
        assert!(stats.total_size_bytes > 0);
    }

    #[test]
    fn test_cleanup_old_backups() {
        let temp_dir = TempDir::new().unwrap();
        let backup_root = temp_dir.path().join("backups");
        let source_dir = temp_dir.path().join("source");

        fs::create_dir_all(&source_dir).unwrap();
        fs::write(source_dir.join("test.txt"), b"Test").unwrap();

        let manager = BackupManager::new(backup_root).unwrap();

        for i in 0..5 {
            manager.create_full_backup(&source_dir, &format!("backup{}", i)).unwrap();
        }

        assert_eq!(manager.list_backups().len(), 5);

        let deleted = manager.cleanup_old_backups(3).unwrap();
        assert_eq!(deleted, 2);
        assert_eq!(manager.list_backups().len(), 3);
    }
}
