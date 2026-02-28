use std::sync::atomic::{AtomicU64, Ordering};

static TRACE_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceContext {
    pub trace_id: String,
    pub request_id: String,
}

impl TraceContext {
    pub fn new(request_id: impl Into<String>) -> Self {
        Self {
            trace_id: next_id("trace"),
            request_id: request_id.into(),
        }
    }

    pub fn generate() -> Self {
        let request_id = next_id("req");
        Self::new(request_id)
    }
}

fn next_id(prefix: &str) -> String {
    let value = TRACE_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}-{:016x}", prefix, value)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEvent {
    pub trace_id: String,
    pub request_id: String,
    pub command_id: String,
    pub duration_ms: u128,
    pub exit_code: i32,
    pub noop: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_context_generates_unique_trace_id() {
        let a = TraceContext::generate();
        let b = TraceContext::generate();
        assert_ne!(a.trace_id, b.trace_id);
        assert_ne!(a.request_id, b.request_id);
    }

    #[test]
    fn trace_context_respects_request_id() {
        let ctx = TraceContext::new("request-123");
        assert_eq!(ctx.request_id, "request-123");
        assert!(ctx.trace_id.starts_with("trace-"));
    }
}
