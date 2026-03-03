use pixelcore_logging::{Logger, Tracer, LogLevel, LogQuery};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

fn main() {
    println!("=== PixelCore Logging and Tracing Demo ===\n");

    // 1. 基础日志功能
    println!("1. Basic Logging");
    let logger = Logger::new();

    logger.info("app".to_string(), "Application started".to_string());
    logger.debug("app".to_string(), "Debug information".to_string());
    logger.warn("app".to_string(), "Warning message".to_string());
    logger.error("app".to_string(), "Error occurred".to_string());

    println!("  Logged {} messages", logger.count());
    println!();

    // 2. 带字段的日志
    println!("2. Structured Logging with Fields");
    let mut fields = HashMap::new();
    fields.insert("user_id".to_string(), "user_123".to_string());
    fields.insert("action".to_string(), "login".to_string());
    fields.insert("ip".to_string(), "192.168.1.100".to_string());

    logger.log_with_fields(
        LogLevel::Info,
        "auth".to_string(),
        "User logged in successfully".to_string(),
        fields,
    );

    println!("  Logged structured message with 3 fields");
    println!();

    // 3. 日志查询
    println!("3. Log Querying");

    // 按级别查询
    let query = LogQuery::new().with_level(LogLevel::Info);
    let info_logs = logger.query(&query);
    println!("  INFO logs: {}", info_logs.len());

    // 按消息内容查询
    let query = LogQuery::new().with_message_contains("Error".to_string());
    let error_logs = logger.query(&query);
    println!("  Logs containing 'Error': {}", error_logs.len());

    // 按目标查询
    let query = LogQuery::new().with_target("auth".to_string());
    let auth_logs = logger.query(&query);
    println!("  Auth logs: {}", auth_logs.len());
    println!();

    // 4. 日志统计
    println!("4. Log Statistics");
    let stats = logger.get_stats();
    println!("  Total logs: {}", stats.total_logs);
    println!("  By level:");
    for (level, count) in &stats.by_level {
        println!("    {}: {}", level, count);
    }
    println!("  By target:");
    for (target, count) in &stats.by_target {
        println!("    {}: {}", target, count);
    }
    println!();

    // 5. 分布式追踪
    println!("5. Distributed Tracing");
    let tracer = Tracer::new();

    // 创建一个 trace
    let trace_id = tracer.start_trace("user_request".to_string());
    println!("  Started trace: {}", trace_id);

    // 创建根 span
    let root_span = tracer.start_span(trace_id, "handle_request".to_string(), None);
    tracer.add_span_field(root_span, "method".to_string(), "POST".to_string());
    tracer.add_span_field(root_span, "path".to_string(), "/api/users".to_string());

    // 模拟一些工作
    thread::sleep(Duration::from_millis(50));

    // 创建子 span - 数据库查询
    let db_span = tracer.start_span(
        trace_id,
        "database_query".to_string(),
        Some(root_span),
    );
    tracer.add_span_field(db_span, "query".to_string(), "SELECT * FROM users".to_string());
    thread::sleep(Duration::from_millis(30));
    tracer.end_span(db_span);

    // 创建子 span - 缓存操作
    let cache_span = tracer.start_span(
        trace_id,
        "cache_operation".to_string(),
        Some(root_span),
    );
    tracer.add_span_field(cache_span, "operation".to_string(), "SET".to_string());
    thread::sleep(Duration::from_millis(10));
    tracer.end_span(cache_span);

    // 结束根 span
    tracer.end_span(root_span);
    tracer.end_trace(trace_id);

    println!("  Trace completed with {} spans", tracer.get_trace(trace_id).unwrap().spans.len());
    println!();

    // 6. 日志与追踪集成
    println!("6. Logging with Trace Context");
    let logger2 = Logger::new();
    let trace_id2 = tracer.start_trace("api_call".to_string());
    let span_id2 = tracer.start_span(trace_id2, "process_data".to_string(), None);

    // 设置当前追踪上下文
    logger2.set_current_trace(Some(trace_id2));
    logger2.set_current_span(Some(span_id2));

    // 记录日志（会自动包含 trace_id 和 span_id）
    logger2.info("api".to_string(), "Processing data".to_string());
    logger2.info("api".to_string(), "Data processed successfully".to_string());

    tracer.end_span(span_id2);
    tracer.end_trace(trace_id2);

    // 查询带追踪上下文的日志
    let query = LogQuery::new().with_trace_id(trace_id2);
    let trace_logs = logger2.query(&query);
    println!("  Logs with trace context: {}", trace_logs.len());
    println!();

    // 7. 性能分析
    println!("7. Performance Analysis");
    let trace = tracer.get_trace(trace_id).unwrap();
    println!("  Trace: {}", trace.name);
    println!("  Total duration: {} ms", trace.duration_ms.unwrap_or(0));
    println!("  Spans:");
    for span in &trace.spans {
        let indent = if span.parent_id.is_some() { "    " } else { "  " };
        println!(
            "{}  - {}: {} ms",
            indent,
            span.name,
            span.duration_ms.unwrap_or(0)
        );
    }
    println!();

    // 8. JSON 导出
    println!("8. JSON Export");
    let logs = logger.get_all();
    if let Some(log) = logs.first() {
        println!("  Sample log as JSON:");
        println!("  {}", log.to_json());
    }
    println!();

    // 9. 清理
    println!("9. Cleanup");
    let before_count = logger.count();
    logger.clear();
    let after_count = logger.count();
    println!("  Cleared {} logs (before: {}, after: {})", before_count, before_count, after_count);

    let before_traces = tracer.count_traces();
    tracer.clear();
    let after_traces = tracer.count_traces();
    println!("  Cleared {} traces (before: {}, after: {})", before_traces, before_traces, after_traces);
    println!();

    println!("=== Demo Complete ===");
}
