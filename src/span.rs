use crate::attribute::{ToValue, Value};
use std::collections::HashMap;

/// Represents a distributed tracing span.
#[derive(serde::Serialize, Debug, PartialEq)]
pub struct Span {
    id: String,

    #[serde(rename = "trace.id")]
    trace_id: String,

    timestamp: u64,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    attributes: HashMap<String, Value>,
}

impl Span {
    /// Create a new span and assign an unique identifier, trace id and timestamp
    pub fn new(id: &str, trace_id: &str, timestamp: u64) -> Span {
        Span {
            id: id.to_string(),
            trace_id: trace_id.to_string(),
            timestamp: timestamp,
            attributes: HashMap::new(),
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

    /// Set the start time of the span. This is a required field.
    pub fn timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = timestamp;
        self
    }

    pub fn set_timestamp(&mut self, timestamp: u64) {
        self.timestamp = timestamp;
    }

    /// Set the name of this span.
    pub fn name(self, name: &str) -> Self {
        self.attribute("name", name)
    }

    pub fn set_name(&mut self, name: &str) {
        self.set_attribute("name", name);
    }

    /// Set the duration (in milliseconds) of this span.
    pub fn duration(self, duration: u64) -> Self {
        self.attribute("duration.ms", duration)
    }

    pub fn set_duration(&mut self, duration: u64) {
        self.set_attribute("duration.ms", duration);
    }

    /// Set the id of the previous caller of this span.
    pub fn parent_id(self, parent_id: &str) -> Self {
        self.attribute("parent.id", parent_id)
    }

    pub fn set_parent_id(&mut self, parent_id: &str) {
        self.set_attribute("parent.id", parent_id);
    }

    /// Set the name of the service that created this span.
    pub fn service_name(self, service_name: &str) -> Self {
        self.attribute("service.name", service_name)
    }

    pub fn set_service_name(&mut self, service_name: &str) {
        self.set_attribute("service.name", service_name);
    }

    /// Set an attribute on the span.
    pub fn attribute<T: ToValue>(mut self, key: &str, value: T) -> Self {
        self.attributes
            .insert(key.to_string(), value.to_attribute_value());
        self
    }

    pub fn set_attribute<T: ToValue>(&mut self, key: &str, value: T) {
        self.attributes
            .insert(key.to_string(), value.to_attribute_value());
    }
}

#[cfg(test)]
mod tests {
    use super::Span;
    use crate::attribute::Value;
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
        // Check span JSON serialization with empty attribute hashmap.
        let mut span = Span::new("id1", "traceId1", 1);
        let json_span = json!({"id": "id1", "trace.id": "traceId1", "timestamp": 1});

        assert_eq!(json!(span), json_span);

        // Check span JSON serialization with populated attribute hashmap.
        span.set_name("I have a name");
        let json_span_attribute = json!({"id": "id1", "trace.id": "traceId1", "timestamp": 1, "attributes": {"name": "I have a name"}});

        assert_eq!(json!(span), json_span_attribute);
    }

    #[test]
    fn test_attribute() {
        let mut span = Span::new("id1", "traceId1", 1);

        // Test name attribute
        span.set_name("name");
        assert_eq!(
            span.attributes.get("name"),
            Some(&Value::Str(String::from("name")))
        );

        span = span.name("name2");
        assert_eq!(
            span.attributes.get("name"),
            Some(&Value::Str(String::from("name2")))
        );

        // Test duration attribute
        span.set_duration(1);
        assert_eq!(span.attributes.get("duration.ms"), Some(&Value::UInt(1)));

        span = span.duration(2);
        assert_eq!(span.attributes.get("duration.ms"), Some(&Value::UInt(2)));

        // Test parent id attribute
        span.set_parent_id("parent");
        assert_eq!(
            span.attributes.get("parent.id"),
            Some(&Value::Str(String::from("parent")))
        );

        span = span.parent_id("parent2");
        assert_eq!(
            span.attributes.get("parent.id"),
            Some(&Value::Str(String::from("parent2")))
        );

        // Test service name attribute
        span.set_service_name("serviceName");
        assert_eq!(
            span.attributes.get("service.name"),
            Some(&Value::Str(String::from("serviceName")))
        );

        span = span.service_name("serviceName2");
        assert_eq!(
            span.attributes.get("service.name"),
            Some(&Value::Str(String::from("serviceName2")))
        );

        // Test parent id attribute
        span.set_attribute("attribute.one", "attribute1");
        assert_eq!(
            span.attributes.get("attribute.one"),
            Some(&Value::Str(String::from("attribute1")))
        );

        span = span.attribute("attribute.two", "attribute2");
        assert_eq!(
            span.attributes.get("attribute.two"),
            Some(&Value::Str(String::from("attribute2")))
        );
    }
}
