use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

/// 日志记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRecord {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub target: String,
    pub message: String,
    pub fields: HashMap<String, String>,
    pub span_id: Option<Uuid>,
    pub trace_id: Option<Uuid>,
}

impl LogRecord {
    pub fn new(level: LogLevel, target: String, message: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            level,
            target,
            message,
            fields: HashMap::new(),
            span_id: None,
            trace_id: None,
        }
    }

    pub fn with_field(mut self, key: String, value: String) -> Self {
        self.fields.insert(key, value);
        self
    }

    pub fn with_span(mut self, span_id: Uuid) -> Self {
        self.span_id = Some(span_id);
        self
    }

    pub fn with_trace(mut self, trace_id: Uuid) -> Self {
        self.trace_id = Some(trace_id);
        self
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Span (追踪单元)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    pub id: Uuid,
    pub trace_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub fields: HashMap<String, String>,
    pub events: Vec<SpanEvent>,
}

impl Span {
    pub fn new(trace_id: Uuid, name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            trace_id,
            parent_id: None,
            name,
            start_time: Utc::now(),
            end_time: None,
            duration_ms: None,
            fields: HashMap::new(),
            events: Vec::new(),
        }
    }

    pub fn with_parent(mut self, parent_id: Uuid) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    pub fn with_field(mut self, key: String, value: String) -> Self {
        self.fields.insert(key, value);
        self
    }

    pub fn add_event(&mut self, name: String, fields: HashMap<String, String>) {
        self.events.push(SpanEvent {
            timestamp: Utc::now(),
            name,
            fields,
        });
    }

    pub fn finish(&mut self) {
        let end_time = Utc::now();
        self.end_time = Some(end_time);
        self.duration_ms = Some((end_time - self.start_time).num_milliseconds() as u64);
    }
}

/// Span 事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
    pub timestamp: DateTime<Utc>,
    pub name: String,
    pub fields: HashMap<String, String>,
}

/// Trace (完整的调用链)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    pub id: Uuid,
    pub name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub spans: Vec<Span>,
    pub root_span_id: Option<Uuid>,
}

impl Trace {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            start_time: Utc::now(),
            end_time: None,
            duration_ms: None,
            spans: Vec::new(),
            root_span_id: None,
        }
    }

    pub fn add_span(&mut self, span: Span) {
        if self.root_span_id.is_none() && span.parent_id.is_none() {
            self.root_span_id = Some(span.id);
        }
        self.spans.push(span);
    }

    pub fn finish(&mut self) {
        let end_time = Utc::now();
        self.end_time = Some(end_time);
        self.duration_ms = Some((end_time - self.start_time).num_milliseconds() as u64);
    }

    pub fn get_span(&self, span_id: Uuid) -> Option<&Span> {
        self.spans.iter().find(|s| s.id == span_id)
    }
}

/// 日志查询条件
#[derive(Debug, Clone, Default)]
pub struct LogQuery {
    pub level: Option<LogLevel>,
    pub target: Option<String>,
    pub message_contains: Option<String>,
    pub trace_id: Option<Uuid>,
    pub span_id: Option<Uuid>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}

impl LogQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_level(mut self, level: LogLevel) -> Self {
        self.level = Some(level);
        self
    }

    pub fn with_target(mut self, target: String) -> Self {
        self.target = Some(target);
        self
    }

    pub fn with_message_contains(mut self, text: String) -> Self {
        self.message_contains = Some(text);
        self
    }

    pub fn with_trace_id(mut self, trace_id: Uuid) -> Self {
        self.trace_id = Some(trace_id);
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn matches(&self, record: &LogRecord) -> bool {
        if let Some(level) = self.level {
            if record.level != level {
                return false;
            }
        }

        if let Some(ref target) = self.target {
            if !record.target.contains(target) {
                return false;
            }
        }

        if let Some(ref text) = self.message_contains {
            if !record.message.contains(text) {
                return false;
            }
        }

        if let Some(trace_id) = self.trace_id {
            if record.trace_id != Some(trace_id) {
                return false;
            }
        }

        if let Some(span_id) = self.span_id {
            if record.span_id != Some(span_id) {
                return false;
            }
        }

        if let Some(start_time) = self.start_time {
            if record.timestamp < start_time {
                return false;
            }
        }

        if let Some(end_time) = self.end_time {
            if record.timestamp > end_time {
                return false;
            }
        }

        true
    }
}

/// 日志统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStats {
    pub total_logs: usize,
    pub by_level: HashMap<String, usize>,
    pub by_target: HashMap<String, usize>,
    pub time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
}

impl LogStats {
    pub fn new() -> Self {
        Self {
            total_logs: 0,
            by_level: HashMap::new(),
            by_target: HashMap::new(),
            time_range: None,
        }
    }
}

impl Default for LogStats {
    fn default() -> Self {
        Self::new()
    }
}

