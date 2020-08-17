/// Represents a distributed tracing span.
#[derive(serde::Serialize, Debug, PartialEq)]
pub struct Span {
    id: String,

    #[serde(rename = "trace.id")]
    trace_id: String,

    timestamp: u64,
}

impl Span {
    /// Create a new span and assign an unique identifier, trace id and timestamp
    pub fn new(id: &str, trace_id: &str, timestamp: u64) -> Span {
        Span {
            id: id.to_string(),
            trace_id: trace_id.to_string(),
            timestamp: timestamp,
        }
    }

    /// Set a unique identifier for this span. This is a required field.
    pub fn id(mut self, id: &str) -> Self {
        self.id = id.to_string();
        self
    }

    pub fn set_id(&mut self, id: &str) {
        self.id = id.to_string();
    }

    /// Set a unique identifier shared by all spans within a single trace.
    /// This is a required field.
    pub fn trace_id(mut self, trace_id: &str) -> Self {
        self.trace_id = trace_id.to_string();
        self
    }

    pub fn set_trace_id(&mut self, trace_id: &str) {
        self.trace_id = trace_id.to_string();
    }

    /// Set the start time of the span. If the start time is not set, it will be
    /// set to the current time when the span is recorded.
    pub fn timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = timestamp;
        self
    }

    pub fn set_timestamp(&mut self, timestamp: u64) {
        self.timestamp = timestamp;
    }
}

#[cfg(test)]
mod tests {
    use super::Span;
    use serde_json::json;

    #[test]
    fn test_set_id() {
        let mut span = Span::new("id1", "traceId1", 1);
        assert_eq!(span.id, "id1");

        span.set_id("id2");
        assert_eq!(span.id, "id2");

        span = span.id("id3");
        assert_eq!(span.id, "id3");
    }

    #[test]
    fn test_set_trace_id() {
        let mut span = Span::new("id1", "traceId1", 1);
        assert_eq!(span.trace_id, "traceId1");

        span.set_trace_id("traceId2");
        assert_eq!(span.trace_id, "traceId2");

        span = span.trace_id("traceId3");
        assert_eq!(span.trace_id, "traceId3");
    }

    #[test]
    fn test_set_timestamp() {
        let mut span = Span::new("id1", "traceId1", 1);
        assert_eq!(span.timestamp, 1);

        span.set_timestamp(2);
        assert_eq!(span.timestamp, 2);

        span = span.timestamp(3);
        assert_eq!(span.timestamp, 3);
    }

    #[test]
    fn span_to_json() {
        // Check span JSON serialization.
        let span = Span::new("id1", "traceId1", 1);
        let json_span = json!({"id": "id1", "trace.id": "traceId1", "timestamp": 1});

        assert_eq!(json!(span), json_span);

    }
}
