use pixelcore_backup::{BackupManager, RestoreManager, BackupType};
use std::fs;
use tempfile::TempDir;

fn main() -> std::io::Result<()> {
    println!("=== PixelCore Backup and Recovery Demo ===\n");

    // 创建临时目录用于演示
    let temp_dir = TempDir::new()?;
    let backup_root = temp_dir.path().join("backups");
    let source_dir = temp_dir.path().join("source");
    let restore_dir = temp_dir.path().join("restore");

    // 1. 创建测试数据
    println!("1. Creating Test Data");
    fs::create_dir_all(&source_dir)?;
    fs::write(source_dir.join("config.json"), br#"{"app": "pixelcore", "version": "1.0"}"#)?;
    fs::write(source_dir.join("data.txt"), b"Important data")?;
    fs::create_dir_all(source_dir.join("logs"))?;
    fs::write(source_dir.join("logs/app.log"), b"Log entry 1\nLog entry 2")?;

    println!("  Created 3 files in source directory");
    println!("  Source: {:?}", source_dir);
    println!();

    // 2. 创建备份管理器
    println!("2. Creating Backup Manager");
    let backup_manager = BackupManager::new(backup_root.clone())?;
    println!("  Backup root: {:?}", backup_root);
    println!();

    // 3. 创建全量备份
    println!("3. Creating Full Backup");
    let backup_id = backup_manager.create_full_backup(&source_dir, "pixelcore")?;
    println!("  Backup ID: {}", backup_id);

    let backup_record = backup_manager.get_backup(backup_id).unwrap();
    println!("  Backup type: {}", backup_record.backup_type);
    println!("  Status: {:?}", backup_record.status);
    println!("  File count: {}", backup_record.file_count);
    println!("  Original size: {} bytes", backup_record.size_bytes);
    if let Some(compressed) = backup_record.compressed_size_bytes {
        let ratio = (1.0 - (compressed as f64 / backup_record.size_bytes as f64)) * 100.0;
        println!("  Compressed size: {} bytes ({:.1}% reduction)", compressed, ratio);
    }
    println!("  Duration: {} ms", backup_record.duration_ms.unwrap_or(0));
    println!();

    // 4. 创建更多备份
    println!("4. Creating Additional Backups");
    fs::write(source_dir.join("new_file.txt"), b"New content")?;
    let backup_id2 = backup_manager.create_full_backup(&source_dir, "pixelcore")?;
    println!("  Created second backup: {}", backup_id2);

    fs::write(source_dir.join("another.txt"), b"Another file")?;
    let backup_id3 = backup_manager.create_incremental_backup(&source_dir, "pixelcore")?;
    println!("  Created incremental backup: {}", backup_id3);
    println!();

    // 5. 列出所有备份
    println!("5. Listing All Backups");
    let backups = backup_manager.list_backups();
    println!("  Total backups: {}", backups.len());
    for (i, backup) in backups.iter().enumerate() {
        println!("  {}. {} - {} ({} files, {} bytes)",
            i + 1,
            backup.id,
            backup.backup_type,
            backup.file_count,
            backup.size_bytes
        );
    }
    println!();

    // 6. 验证备份
    println!("6. Verifying Backup");
    let is_valid = backup_manager.verify_backup(backup_id)?;
    println!("  Backup valid: {}", is_valid);

    let verified_record = backup_manager.get_backup(backup_id).unwrap();
    println!("  Status: {:?}", verified_record.status);
    println!("  Checksum: {}", verified_record.checksum.unwrap_or_else(|| "N/A".to_string()));
    println!();

    // 7. 获取备份统计
    println!("7. Backup Statistics");
    let stats = backup_manager.get_stats();
    println!("  Total backups: {}", stats.total_backups);
    println!("  Successful: {}", stats.successful_backups);
    println!("  Failed: {}", stats.failed_backups);
    println!("  Total size: {} bytes", stats.total_size_bytes);
    println!("  Compressed size: {} bytes", stats.total_compressed_size_bytes);
    println!("  Average duration: {} ms", stats.average_duration_ms);
    if let Some(oldest) = stats.oldest_backup {
        println!("  Oldest backup: {}", oldest.format("%Y-%m-%d %H:%M:%S"));
    }
    if let Some(newest) = stats.newest_backup {
        println!("  Newest backup: {}", newest.format("%Y-%m-%d %H:%M:%S"));
    }
    println!();

    // 8. 恢复备份
    println!("8. Restoring Backup");
    let restore_manager = RestoreManager::new();

    let restore_id = restore_manager.restore_backup(
        backup_id,
        &backup_record.backup_path,
        &restore_dir,
    )?;
    println!("  Restore ID: {}", restore_id);

    let restore_record = restore_manager.get_restore(restore_id).unwrap();
    println!("  Status: {:?}", restore_record.status);
    println!("  Files restored: {}", restore_record.file_count);
    println!("  Bytes restored: {}", restore_record.restored_bytes);
    println!("  Duration: {} ms", restore_record.duration_ms.unwrap_or(0));
    println!();

    // 9. 验证恢复
    println!("9. Verifying Restore");
    let is_valid = restore_manager.verify_restore(restore_id)?;
    println!("  Restore valid: {}", is_valid);

    // 检查恢复的文件
    println!("  Restored files:");
    for entry in fs::read_dir(&restore_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            println!("    - {:?}", path.file_name().unwrap());
        } else if path.is_dir() {
            println!("    - {:?}/ (directory)", path.file_name().unwrap());
        }
    }
    println!();

    // 10. 清理旧备份
    println!("10. Cleanup Old Backups");
    println!("  Backups before cleanup: {}", backup_manager.list_backups().len());

    let deleted = backup_manager.cleanup_old_backups(2)?;
    println!("  Deleted {} old backups", deleted);
    println!("  Backups after cleanup: {}", backup_manager.list_backups().len());
    println!();

    // 11. RTO/RPO 演示
    println!("11. RTO/RPO Demonstration");
    println!("  RTO (Recovery Time Objective):");
    println!("    - Backup creation: {} ms", backup_record.duration_ms.unwrap_or(0));
    println!("    - Restore operation: {} ms", restore_record.duration_ms.unwrap_or(0));
    println!("    - Total recovery time: {} ms",
        backup_record.duration_ms.unwrap_or(0) + restore_record.duration_ms.unwrap_or(0));
    println!();
    println!("  RPO (Recovery Point Objective):");
    println!("    - Last backup: {}", backup_record.created_at.format("%Y-%m-%d %H:%M:%S"));
    println!("    - Data loss window: < 1 minute (with continuous backup)");
    println!();

    println!("=== Demo Complete ===");
    println!("\nNote: All files created in temporary directory will be cleaned up automatically.");

    Ok(())
}
