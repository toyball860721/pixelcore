use crate::models::{Span, Trace};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Tracer {
    traces: Arc<Mutex<HashMap<Uuid, Trace>>>,
    active_spans: Arc<Mutex<HashMap<Uuid, Span>>>,
}

impl Tracer {
    pub fn new() -> Self {
        Self {
            traces: Arc::new(Mutex::new(HashMap::new())),
            active_spans: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 创建新的 trace
    pub fn start_trace(&self, name: String) -> Uuid {
        let trace = Trace::new(name);
        let trace_id = trace.id;

        let mut traces = self.traces.lock().unwrap();
        traces.insert(trace_id, trace);

        trace_id
    }

    /// 创建新的 span
    pub fn start_span(&self, trace_id: Uuid, name: String, parent_id: Option<Uuid>) -> Uuid {
        let mut span = Span::new(trace_id, name);
        if let Some(parent) = parent_id {
            span = span.with_parent(parent);
        }

        let span_id = span.id;

        let mut active_spans = self.active_spans.lock().unwrap();
        active_spans.insert(span_id, span);

        span_id
    }

    /// 添加 span 字段
    pub fn add_span_field(&self, span_id: Uuid, key: String, value: String) {
        let mut active_spans = self.active_spans.lock().unwrap();
        if let Some(span) = active_spans.get_mut(&span_id) {
            span.fields.insert(key, value);
        }
    }

    /// 添加 span 事件
    pub fn add_span_event(&self, span_id: Uuid, name: String, fields: HashMap<String, String>) {
        let mut active_spans = self.active_spans.lock().unwrap();
        if let Some(span) = active_spans.get_mut(&span_id) {
            span.add_event(name, fields);
        }
    }

    /// 结束 span
    pub fn end_span(&self, span_id: Uuid) {
        let mut active_spans = self.active_spans.lock().unwrap();
        if let Some(mut span) = active_spans.remove(&span_id) {
            span.finish();

            let trace_id = span.trace_id;
            drop(active_spans); // 释放锁

            let mut traces = self.traces.lock().unwrap();
            if let Some(trace) = traces.get_mut(&trace_id) {
                trace.add_span(span);
            }
        }
    }

    /// 结束 trace
    pub fn end_trace(&self, trace_id: Uuid) {
        let mut traces = self.traces.lock().unwrap();
        if let Some(trace) = traces.get_mut(&trace_id) {
            trace.finish();
        }
    }

    /// 获取 trace
    pub fn get_trace(&self, trace_id: Uuid) -> Option<Trace> {
        let traces = self.traces.lock().unwrap();
        traces.get(&trace_id).cloned()
    }

    /// 获取所有 traces
    pub fn get_all_traces(&self) -> Vec<Trace> {
        let traces = self.traces.lock().unwrap();
        traces.values().cloned().collect()
    }

    /// 获取 trace 数量
    pub fn count_traces(&self) -> usize {
        let traces = self.traces.lock().unwrap();
        traces.len()
    }

    /// 获取活跃 span 数量
    pub fn count_active_spans(&self) -> usize {
        let active_spans = self.active_spans.lock().unwrap();
        active_spans.len()
    }

    /// 清空所有数据
    pub fn clear(&self) {
        let mut traces = self.traces.lock().unwrap();
        traces.clear();

        let mut active_spans = self.active_spans.lock().unwrap();
        active_spans.clear();
    }
}

impl Default for Tracer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_trace() {
        let tracer = Tracer::new();

        let trace_id = tracer.start_trace("test_trace".to_string());
        assert_eq!(tracer.count_traces(), 1);

        let trace = tracer.get_trace(trace_id).unwrap();
        assert_eq!(trace.name, "test_trace");
        assert_eq!(trace.id, trace_id);
    }

    #[test]
    fn test_create_span() {
        let tracer = Tracer::new();

        let trace_id = tracer.start_trace("test_trace".to_string());
        let span_id = tracer.start_span(trace_id, "test_span".to_string(), None);

        assert_eq!(tracer.count_active_spans(), 1);

        tracer.end_span(span_id);
        assert_eq!(tracer.count_active_spans(), 0);

        let trace = tracer.get_trace(trace_id).unwrap();
        assert_eq!(trace.spans.len(), 1);
        assert_eq!(trace.spans[0].name, "test_span");
    }

    #[test]
    fn test_nested_spans() {
        let tracer = Tracer::new();

        let trace_id = tracer.start_trace("test_trace".to_string());
        let parent_span_id = tracer.start_span(trace_id, "parent_span".to_string(), None);
        let child_span_id = tracer.start_span(
            trace_id,
            "child_span".to_string(),
            Some(parent_span_id),
        );

        assert_eq!(tracer.count_active_spans(), 2);

        tracer.end_span(child_span_id);
        tracer.end_span(parent_span_id);

        let trace = tracer.get_trace(trace_id).unwrap();
        assert_eq!(trace.spans.len(), 2);

        // 验证父子关系
        let child = trace.spans.iter().find(|s| s.name == "child_span").unwrap();
        assert_eq!(child.parent_id, Some(parent_span_id));
    }

    #[test]
    fn test_span_fields() {
        let tracer = Tracer::new();

        let trace_id = tracer.start_trace("test_trace".to_string());
        let span_id = tracer.start_span(trace_id, "test_span".to_string(), None);

        tracer.add_span_field(span_id, "user_id".to_string(), "123".to_string());
        tracer.add_span_field(span_id, "action".to_string(), "login".to_string());

        tracer.end_span(span_id);

        let trace = tracer.get_trace(trace_id).unwrap();
        let span = &trace.spans[0];

        assert_eq!(span.fields.get("user_id").unwrap(), "123");
        assert_eq!(span.fields.get("action").unwrap(), "login");
    }

    #[test]
    fn test_span_events() {
        let tracer = Tracer::new();

        let trace_id = tracer.start_trace("test_trace".to_string());
        let span_id = tracer.start_span(trace_id, "test_span".to_string(), None);

        let mut fields = HashMap::new();
        fields.insert("status".to_string(), "success".to_string());

        tracer.add_span_event(span_id, "checkpoint".to_string(), fields);

        tracer.end_span(span_id);

        let trace = tracer.get_trace(trace_id).unwrap();
        let span = &trace.spans[0];

        assert_eq!(span.events.len(), 1);
        assert_eq!(span.events[0].name, "checkpoint");
        assert_eq!(span.events[0].fields.get("status").unwrap(), "success");
    }

    #[test]
    fn test_span_duration() {
        let tracer = Tracer::new();

        let trace_id = tracer.start_trace("test_trace".to_string());
        let span_id = tracer.start_span(trace_id, "test_span".to_string(), None);

        // 模拟一些工作
        std::thread::sleep(std::time::Duration::from_millis(10));

        tracer.end_span(span_id);

        let trace = tracer.get_trace(trace_id).unwrap();
        let span = &trace.spans[0];

        assert!(span.duration_ms.is_some());
        assert!(span.duration_ms.unwrap() >= 10);
    }

    #[test]
    fn test_trace_finish() {
        let tracer = Tracer::new();

        let trace_id = tracer.start_trace("test_trace".to_string());
        let span_id = tracer.start_span(trace_id, "test_span".to_string(), None);

        tracer.end_span(span_id);
        tracer.end_trace(trace_id);

        let trace = tracer.get_trace(trace_id).unwrap();
        assert!(trace.end_time.is_some());
        assert!(trace.duration_ms.is_some());
    }
}
