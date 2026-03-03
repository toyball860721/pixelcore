use crate::models::{LogLevel, LogRecord, LogQuery, LogStats};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Logger {
    records: Arc<Mutex<Vec<LogRecord>>>,
    current_span_id: Arc<Mutex<Option<Uuid>>>,
    current_trace_id: Arc<Mutex<Option<Uuid>>>,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            records: Arc::new(Mutex::new(Vec::new())),
            current_span_id: Arc::new(Mutex::new(None)),
            current_trace_id: Arc::new(Mutex::new(None)),
        }
    }

    /// 设置当前 span
    pub fn set_current_span(&self, span_id: Option<Uuid>) {
        let mut current = self.current_span_id.lock().unwrap();
        *current = span_id;
    }

    /// 设置当前 trace
    pub fn set_current_trace(&self, trace_id: Option<Uuid>) {
        let mut current = self.current_trace_id.lock().unwrap();
        *current = trace_id;
    }

    /// 记录日志
    pub fn log(&self, level: LogLevel, target: String, message: String) {
        let mut record = LogRecord::new(level, target, message);

        // 添加当前 span 和 trace
        if let Some(span_id) = *self.current_span_id.lock().unwrap() {
            record.span_id = Some(span_id);
        }
        if let Some(trace_id) = *self.current_trace_id.lock().unwrap() {
            record.trace_id = Some(trace_id);
        }

        let mut records = self.records.lock().unwrap();
        records.push(record);
    }

    /// 记录带字段的日志
    pub fn log_with_fields(
        &self,
        level: LogLevel,
        target: String,
        message: String,
        fields: HashMap<String, String>,
    ) {
        let mut record = LogRecord::new(level, target, message);
        record.fields = fields;

        // 添加当前 span 和 trace
        if let Some(span_id) = *self.current_span_id.lock().unwrap() {
            record.span_id = Some(span_id);
        }
        if let Some(trace_id) = *self.current_trace_id.lock().unwrap() {
            record.trace_id = Some(trace_id);
        }

        let mut records = self.records.lock().unwrap();
        records.push(record);
    }

    /// Trace 级别日志
    pub fn trace(&self, target: String, message: String) {
        self.log(LogLevel::Trace, target, message);
    }

    /// Debug 级别日志
    pub fn debug(&self, target: String, message: String) {
        self.log(LogLevel::Debug, target, message);
    }

    /// Info 级别日志
    pub fn info(&self, target: String, message: String) {
        self.log(LogLevel::Info, target, message);
    }

    /// Warn 级别日志
    pub fn warn(&self, target: String, message: String) {
        self.log(LogLevel::Warn, target, message);
    }

    /// Error 级别日志
    pub fn error(&self, target: String, message: String) {
        self.log(LogLevel::Error, target, message);
    }

    /// 查询日志
    pub fn query(&self, query: &LogQuery) -> Vec<LogRecord> {
        let records = self.records.lock().unwrap();
        let mut results: Vec<LogRecord> = records
            .iter()
            .filter(|r| query.matches(r))
            .cloned()
            .collect();

        // 按时间倒序排序
        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // 应用限制
        if let Some(limit) = query.limit {
            results.truncate(limit);
        }

        results
    }

    /// 获取所有日志
    pub fn get_all(&self) -> Vec<LogRecord> {
        let records = self.records.lock().unwrap();
        records.clone()
    }

    /// 获取日志统计
    pub fn get_stats(&self) -> LogStats {
        let records = self.records.lock().unwrap();
        let mut stats = LogStats::new();

        stats.total_logs = records.len();

        for record in records.iter() {
            // 按级别统计
            let level_key = record.level.to_string();
            *stats.by_level.entry(level_key).or_insert(0) += 1;

            // 按目标统计
            *stats.by_target.entry(record.target.clone()).or_insert(0) += 1;

            // 更新时间范围
            if let Some((start, end)) = stats.time_range {
                stats.time_range = Some((
                    start.min(record.timestamp),
                    end.max(record.timestamp),
                ));
            } else {
                stats.time_range = Some((record.timestamp, record.timestamp));
            }
        }

        stats
    }

    /// 清空日志
    pub fn clear(&self) {
        let mut records = self.records.lock().unwrap();
        records.clear();
    }

    /// 获取日志数量
    pub fn count(&self) -> usize {
        let records = self.records.lock().unwrap();
        records.len()
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_basic() {
        let logger = Logger::new();

        logger.info("test".to_string(), "Hello, world!".to_string());
        logger.warn("test".to_string(), "Warning message".to_string());
        logger.error("test".to_string(), "Error message".to_string());

        assert_eq!(logger.count(), 3);

        let logs = logger.get_all();
        assert_eq!(logs[0].level, LogLevel::Info);
        assert_eq!(logs[1].level, LogLevel::Warn);
        assert_eq!(logs[2].level, LogLevel::Error);
    }

    #[test]
    fn test_log_with_fields() {
        let logger = Logger::new();

        let mut fields = HashMap::new();
        fields.insert("user_id".to_string(), "123".to_string());
        fields.insert("action".to_string(), "login".to_string());

        logger.log_with_fields(
            LogLevel::Info,
            "auth".to_string(),
            "User logged in".to_string(),
            fields,
        );

        let logs = logger.get_all();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].fields.get("user_id").unwrap(), "123");
        assert_eq!(logs[0].fields.get("action").unwrap(), "login");
    }

    #[test]
    fn test_query_by_level() {
        let logger = Logger::new();

        logger.info("test".to_string(), "Info 1".to_string());
        logger.warn("test".to_string(), "Warn 1".to_string());
        logger.error("test".to_string(), "Error 1".to_string());
        logger.info("test".to_string(), "Info 2".to_string());

        let query = LogQuery::new().with_level(LogLevel::Info);
        let results = logger.query(&query);

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.level == LogLevel::Info));
    }

    #[test]
    fn test_query_by_message() {
        let logger = Logger::new();

        logger.info("test".to_string(), "User logged in".to_string());
        logger.info("test".to_string(), "User logged out".to_string());
        logger.info("test".to_string(), "System started".to_string());

        let query = LogQuery::new().with_message_contains("logged".to_string());
        let results = logger.query(&query);

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.message.contains("logged")));
    }

    #[test]
    fn test_stats() {
        let logger = Logger::new();

        logger.info("auth".to_string(), "Message 1".to_string());
        logger.info("auth".to_string(), "Message 2".to_string());
        logger.warn("api".to_string(), "Message 3".to_string());
        logger.error("db".to_string(), "Message 4".to_string());

        let stats = logger.get_stats();

        assert_eq!(stats.total_logs, 4);
        assert_eq!(*stats.by_level.get("INFO").unwrap(), 2);
        assert_eq!(*stats.by_level.get("WARN").unwrap(), 1);
        assert_eq!(*stats.by_level.get("ERROR").unwrap(), 1);
        assert_eq!(*stats.by_target.get("auth").unwrap(), 2);
        assert_eq!(*stats.by_target.get("api").unwrap(), 1);
        assert_eq!(*stats.by_target.get("db").unwrap(), 1);
    }

    #[test]
    fn test_with_trace_context() {
        let logger = Logger::new();
        let trace_id = Uuid::new_v4();
        let span_id = Uuid::new_v4();

        logger.set_current_trace(Some(trace_id));
        logger.set_current_span(Some(span_id));

        logger.info("test".to_string(), "Message with context".to_string());

        let logs = logger.get_all();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].trace_id, Some(trace_id));
        assert_eq!(logs[0].span_id, Some(span_id));
    }
}
