//! Distributed tracing for ultra-low latency systems

use crate::error::{LoggingError, Result};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TraceId(pub u128);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpanId(pub u64);

static SPAN_COUNTER: AtomicU64 = AtomicU64::new(1);

impl TraceId {
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        Self(now)
    }
    
    pub fn from_u128(id: u128) -> Self {
        Self(id)
    }
    
    pub fn as_u128(&self) -> u128 {
        self.0
    }
    
    pub fn to_hex_string(&self) -> String {
        format!("{:032x}", self.0)
    }
}

impl SpanId {
    pub fn new() -> Self {
        Self(SPAN_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
    
    pub fn from_u64(id: u64) -> Self {
        Self(id)
    }
    
    pub fn as_u64(&self) -> u64 {
        self.0
    }
    
    pub fn to_hex_string(&self) -> String {
        format!("{:016x}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct TraceContext {
    pub trace_id: TraceId,
    pub span_id: SpanId,
    pub parent_span_id: Option<SpanId>,
    pub baggage: HashMap<String, String>,
}

impl TraceContext {
    pub fn new() -> Self {
        Self {
            trace_id: TraceId::new(),
            span_id: SpanId::new(),
            parent_span_id: None,
            baggage: HashMap::new(),
        }
    }
    
    pub fn child_span(&self) -> Self {
        Self {
            trace_id: self.trace_id.clone(),
            span_id: SpanId::new(),
            parent_span_id: Some(self.span_id.clone()),
            baggage: self.baggage.clone(),
        }
    }
    
    pub fn with_baggage_item(mut self, key: String, value: String) -> Self {
        self.baggage.insert(key, value);
        self
    }
    
    pub fn get_baggage_item(&self, key: &str) -> Option<&String> {
        self.baggage.get(key)
    }
}

impl Default for TraceContext {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Span {
    pub context: TraceContext,
    pub operation_name: String,
    pub start_time: u64, // microseconds since epoch
    pub end_time: Option<u64>,
    pub tags: HashMap<String, String>,
    pub logs: Vec<SpanLog>,
}

#[derive(Debug, Clone)]
pub struct SpanLog {
    pub timestamp: u64, // microseconds since epoch
    pub fields: HashMap<String, String>,
}

impl Span {
    pub fn new(operation_name: String) -> Self {
        Self {
            context: TraceContext::new(),
            operation_name,
            start_time: current_time_micros(),
            end_time: None,
            tags: HashMap::new(),
            logs: Vec::new(),
        }
    }
    
    pub fn child_span(&self, operation_name: String) -> Self {
        Self {
            context: self.context.child_span(),
            operation_name,
            start_time: current_time_micros(),
            end_time: None,
            tags: HashMap::new(),
            logs: Vec::new(),
        }
    }
    
    pub fn set_tag(mut self, key: String, value: String) -> Self {
        self.tags.insert(key, value);
        self
    }
    
    pub fn log_event(mut self, message: String) -> Self {
        let mut fields = HashMap::new();
        fields.insert("event".to_string(), message);
        self.logs.push(SpanLog {
            timestamp: current_time_micros(),
            fields,
        });
        self
    }
    
    pub fn log_with_fields(mut self, fields: HashMap<String, String>) -> Self {
        self.logs.push(SpanLog {
            timestamp: current_time_micros(),
            fields,
        });
        self
    }
    
    pub fn finish(mut self) -> Self {
        self.end_time = Some(current_time_micros());
        self
    }
    
    pub fn duration_micros(&self) -> Option<u64> {
        self.end_time.map(|end| end - self.start_time)
    }
    
    pub fn is_finished(&self) -> bool {
        self.end_time.is_some()
    }
}

// Trading-specific span builders
impl Span {
    pub fn trade_execution(symbol: &str, side: &str, quantity: f64) -> Self {
        Self::new("trade_execution".to_string())
            .set_tag("symbol".to_string(), symbol.to_string())
            .set_tag("side".to_string(), side.to_string())
            .set_tag("quantity".to_string(), quantity.to_string())
    }
    
    pub fn order_processing(order_id: &str, order_type: &str) -> Self {
        Self::new("order_processing".to_string())
            .set_tag("order_id".to_string(), order_id.to_string())
            .set_tag("order_type".to_string(), order_type.to_string())
    }
    
    pub fn market_data_processing(symbol: &str, data_type: &str) -> Self {
        Self::new("market_data_processing".to_string())
            .set_tag("symbol".to_string(), symbol.to_string())
            .set_tag("data_type".to_string(), data_type.to_string())
    }
    
    pub fn risk_check(position_id: &str, check_type: &str) -> Self {
        Self::new("risk_check".to_string())
            .set_tag("position_id".to_string(), position_id.to_string())
            .set_tag("check_type".to_string(), check_type.to_string())
    }
}

pub trait TraceReporter: Send + Sync {
    fn report_span(&self, span: Span) -> Result<()>;
    fn flush(&self) -> Result<()>;
}

pub struct ConsoleTraceReporter;

impl TraceReporter for ConsoleTraceReporter {
    fn report_span(&self, span: Span) -> Result<()> {
        println!(
            "[TRACE] {} ({}ms) trace={} span={} parent={:?}",
            span.operation_name,
            span.duration_micros().unwrap_or(0) / 1000, // Convert to milliseconds
            span.context.trace_id.to_hex_string(),
            span.context.span_id.to_hex_string(),
            span.context.parent_span_id.as_ref().map(|id| id.to_hex_string())
        );
        
        for (key, value) in &span.tags {
            println!("  tag.{}={}", key, value);
        }
        
        for log in &span.logs {
            println!("  log@{}μs:", log.timestamp);
            for (key, value) in &log.fields {
                println!("    {}={}", key, value);
            }
        }
        
        Ok(())
    }
    
    fn flush(&self) -> Result<()> {
        Ok(())
    }
}

thread_local! {
    static CURRENT_SPAN: std::cell::RefCell<Option<Span>> = std::cell::RefCell::new(None);
}

pub struct TracingContext;

impl TracingContext {
    pub fn start_span(operation_name: String) -> Span {
        CURRENT_SPAN.with(|current| {
            let span = match current.borrow().as_ref() {
                Some(parent) => parent.child_span(operation_name),
                None => Span::new(operation_name),
            };
            *current.borrow_mut() = Some(span.clone());
            span
        })
    }
    
    pub fn current_span() -> Option<Span> {
        CURRENT_SPAN.with(|current| current.borrow().clone())
    }
    
    pub fn finish_current_span() -> Option<Span> {
        CURRENT_SPAN.with(|current| {
            if let Some(span) = current.borrow_mut().take() {
                let finished = span.finish();
                Some(finished)
            } else {
                None
            }
        })
    }
    
    pub fn with_span<F, R>(span: Span, f: F) -> R 
    where 
        F: FnOnce() -> R 
    {
        CURRENT_SPAN.with(|current| {
            let previous = current.borrow().clone();
            *current.borrow_mut() = Some(span);
            let result = f();
            *current.borrow_mut() = previous;
            result
        })
    }
}

// Utility function to get current time in microseconds
fn current_time_micros() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros() as u64
}

// Macros for convenient tracing
#[macro_export]
macro_rules! trace_span {
    ($name:expr) => {
        $crate::trace::TracingContext::start_span($name.to_string())
    };
    ($name:expr, $($key:expr => $value:expr),*) => {
        {
            let mut span = $crate::trace::TracingContext::start_span($name.to_string());
            $(
                span = span.set_tag($key.to_string(), $value.to_string());
            )*
            span
        }
    };
}

#[macro_export]
macro_rules! trace_event {
    ($message:expr) => {
        if let Some(mut span) = $crate::trace::TracingContext::current_span() {
            span = span.log_event($message.to_string());
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_trace_context() {
        let ctx = TraceContext::new();
        let child_ctx = ctx.child_span();
        
        assert_eq!(ctx.trace_id, child_ctx.trace_id);
        assert_ne!(ctx.span_id, child_ctx.span_id);
        assert_eq!(child_ctx.parent_span_id, Some(ctx.span_id));
    }
    
    #[test]
    fn test_span_lifecycle() {
        let span = Span::new("test_operation".to_string())
            .set_tag("test_key".to_string(), "test_value".to_string())
            .log_event("test event".to_string())
            .finish();
        
        assert!(span.is_finished());
        assert!(span.duration_micros().is_some());
        assert_eq!(span.tags.get("test_key"), Some(&"test_value".to_string()));
        assert_eq!(span.logs.len(), 1);
    }
}
