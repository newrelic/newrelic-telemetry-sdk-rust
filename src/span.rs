use crate::attribute::Value;
use crate::client::Sendable;
use anyhow::Result;
#[cfg(test)]
use serde::{Serialize, Serializer};
#[cfg(test)]
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt;
use std::time::Duration;

/// Represents a distributed tracing span.
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
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
    pub fn duration(self, duration: Duration) -> Self {
        self.attribute("duration.ms", duration.as_millis())
    }

    pub fn set_duration(&mut self, duration: Duration) {
        self.set_attribute("duration.ms", duration.as_millis());
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
    pub fn attribute<T: Into<Value>>(mut self, key: &str, value: T) -> Self {
        self.attributes.insert(key.to_string(), value.into());
        self
    }

    pub fn set_attribute<T: Into<Value>>(&mut self, key: &str, value: T) {
        self.attributes.insert(key.to_string(), value.into());
    }
}

/// Represents the common fields of a SpanBatch.
/// At the moment, only `attributes` is defined.
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
struct SpanBatchCommon {
    // Only serialize if there is data.  If testing, sort data via a BTreeMap
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[cfg_attr(test, serde(serialize_with = "hash_to_btree"))]
    attributes: HashMap<String, Value>,
}

#[cfg(test)]
fn hash_to_btree<S>(value: &HashMap<String, Value>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    value
        .iter()
        .collect::<BTreeMap<_, _>>()
        .serialize(serializer)
}

impl SpanBatchCommon {
    /// Create a new SpanBatchCommon with empty values.
    fn new() -> Self {
        SpanBatchCommon {
            attributes: HashMap::new(),
        }
    }

    /// Adds an attribute to the common attributes map.
    fn set_attribute<T: Into<Value>>(&mut self, key: &str, value: T) {
        self.attributes.insert(key.to_string(), value.into());
    }

    /// Returns true if no common fields have values added to them.
    /// This is primarily used to determine omission during SpanBatch serialization
    fn is_empty(&self) -> bool {
        self.attributes.is_empty()
    }
}

/// Encapsulates a collection of spans and the common data they share
#[derive(serde::Serialize, Debug, PartialEq)]
pub struct SpanBatch {
    spans: Vec<Span>,

    /// A struct containing the common data of the batched spans. At this time,
    /// only `attributes` is supported.
    #[serde(skip_serializing_if = "SpanBatchCommon::is_empty")]
    common: SpanBatchCommon,
}

impl SpanBatch {
    /// Creates a new `SpanBatch` with all collections empty.
    pub fn new() -> Box<Self> {
        Box::new(SpanBatch {
            spans: vec![],
            common: SpanBatchCommon::new(),
        })
    }

    /// Creates a new `SpanBatch` from a `Vec<Span>`
    pub fn from(spans: Vec<Span>) -> Box<Self> {
        let mut batch = Self::new();

        for span in spans {
            batch.record(span);
        }

        batch
    }

    /// Adds the provided span to the batch.
    pub fn record(&mut self, span: Span) {
        self.spans.push(span);
    }

    /// Sets an attribute on the span batch. Returns `self` and can be chained
    /// for concise addition of multiple attributes.
    pub fn attribute<T: Into<Value>>(mut self, key: &str, value: T) -> Box<Self> {
        self.common.set_attribute(key, value);
        Box::new(self)
    }

    /// Sets an attribute on the span batch.
    pub fn set_attribute<T: Into<Value>>(&mut self, key: &str, value: T) {
        self.common.set_attribute(key, value);
    }
}

impl Sendable for SpanBatch {
    /// Returns the span batch encoded as a json string in the format expected
    /// by the New Relic Telemetry API
    fn marshall(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    /// Splits the batch in half.  This is mostly used when the API service
    /// returns a code indicating that the payload is too large.
    fn split(&mut self) -> Box<dyn Sendable> {
        let new_batch_size: usize = self.spans.len() / 2;

        Box::new(SpanBatch {
            spans: self.spans.drain(new_batch_size..).collect(),
            common: self.common.clone(),
        })
    }
}

impl fmt::Display for SpanBatch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<SpanBatch spans:{} attributes:{}>",
            self.spans.len(),
            self.common.attributes.len(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{Sendable, Span, SpanBatch};
    use crate::attribute::Value;
    use serde_json::json;
    use std::time::Duration;

    #[test]
    fn span_set_id() {
        let mut span = Span::new("id1", "traceId1", 1);
        assert_eq!(span.id, "id1");

        span.set_id("id2");
        assert_eq!(span.id, "id2");

        span = span.id("id3");
        assert_eq!(span.id, "id3");
    }

    #[test]
    fn span_set_trace_id() {
        let mut span = Span::new("id1", "traceId1", 1);
        assert_eq!(span.trace_id, "traceId1");

        span.set_trace_id("traceId2");
        assert_eq!(span.trace_id, "traceId2");

        span = span.trace_id("traceId3");
        assert_eq!(span.trace_id, "traceId3");
    }

    #[test]
    fn span_set_timestamp() {
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
    fn span_attribute() {
        let mut span = Span::new("id", "traceId", 1);

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
        span.set_duration(Duration::from_millis(10));
        assert_eq!(
            span.attributes.get("duration.ms"),
            Some(&Value::UInt128(10))
        );

        span = span.duration(Duration::from_millis(20));
        assert_eq!(
            span.attributes.get("duration.ms"),
            Some(&Value::UInt128(20))
        );

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
    }

    #[test]
    fn span_attribute_type() {
        let mut span = Span::new("id", "traceId", 1);

        // Test String attribute
        span.set_attribute("attr.str", "str");
        assert_eq!(
            span.attributes.get("attr.str"),
            Some(&Value::Str(String::from("str")))
        );

        span = span.attribute("attr.str", "str2");
        assert_eq!(
            span.attributes.get("attr.str"),
            Some(&Value::Str(String::from("str2")))
        );

        // Test UInt attribute
        let val_u32: u32 = 5;
        span.set_attribute("attr.uint", val_u32);
        assert_eq!(
            span.attributes.get("attr.uint"),
            Some(&Value::UInt(val_u32 as u64))
        );

        let val_u64: u64 = 42;
        span = span.attribute("attr.uint", val_u64);
        assert_eq!(
            span.attributes.get("attr.uint"),
            Some(&Value::UInt(val_u64))
        );

        // Test Int attribute
        let val_i32: i32 = -5;
        span.set_attribute("attr.int", val_i32);
        assert_eq!(
            span.attributes.get("attr.int"),
            Some(&Value::Int(val_i32 as i64))
        );

        let val_i64: i64 = -42;
        span = span.attribute("attr.int", val_i64);
        assert_eq!(span.attributes.get("attr.int"), Some(&Value::Int(val_i64)));

        // Test Float attribute
        let val_f32: f32 = 3.14;
        span.set_attribute("attr.float", val_f32);
        assert_eq!(
            span.attributes.get("attr.float"),
            Some(&Value::Float(val_f32 as f64))
        );

        let val_f64: f64 = 6.28;
        span = span.attribute("attr.float", val_f64);
        assert_eq!(
            span.attributes.get("attr.float"),
            Some(&Value::Float(val_f64))
        );

        // Test Bool attribute
        span.set_attribute("attr.bool", true);
        assert_eq!(span.attributes.get("attr.bool"), Some(&Value::Bool(true)));

        span = span.attribute("attr.bool", false);
        assert_eq!(span.attributes.get("attr.bool"), Some(&Value::Bool(false)));
    }

    /// Helper function to generate a simple SpanBatch
    fn span_vec(count: usize) -> Vec<Span> {
        let mut vec = Vec::new();

        for n in 0..count {
            let id = format!("id{}", n);
            let trace_id = format!("trace_id{}", n);
            vec.push(Span::new(id.as_str(), trace_id.as_str(), 1))
        }

        vec
    }

    #[test]
    fn spanbatch_split_partial() {
        // Note: since SpanBatch::split() returns a Box<dyn Sendable>,
        // we cannot fully test split with regard to the returned
        // SpanBatch, only that the originally was drained as expected
        // However, the integration tests cover both sides of this case.
        let mut batch = SpanBatch::from(span_vec(2));
        let _second_batch = batch.split();

        assert_eq!(batch.spans.len(), 1);
        assert_eq!(batch.spans[0], Span::new("id0", "trace_id0", 1));
    }

    #[test]
    fn spanbatch_to_json() {
        // Check span JSON serialization with empty attribute hashmap.
        let batch = SpanBatch::from(span_vec(2)).attribute("attr.test", 3);

        // json! macro imposes a sort which is different from the serde-derive
        // specified order, therefore a string is used
        let expected_string = "{\"spans\":[\
                {\"id\":\"id0\",\"trace.id\":\"trace_id0\",\"timestamp\":1},\
                {\"id\":\"id1\",\"trace.id\":\"trace_id1\",\"timestamp\":1}],\
                \"common\":{\"attributes\":{\"attr.test\":3}}}";

        assert_eq!(batch.marshall().unwrap(), expected_string);
    }

    #[test]
    fn spanbatch_attribute_type() {
        let mut batch = SpanBatch::new();

        // Test String attribute
        batch.set_attribute("attr.str", "str");
        assert_eq!(
            batch.common.attributes.get("attr.str"),
            Some(&Value::Str(String::from("str")))
        );

        batch = batch.attribute("attr.str", "str2");
        assert_eq!(
            batch.common.attributes.get("attr.str"),
            Some(&Value::Str(String::from("str2")))
        );

        // Test UInt attribute
        let val_u32: u32 = 5;
        batch.set_attribute("attr.uint", val_u32);
        assert_eq!(
            batch.common.attributes.get("attr.uint"),
            Some(&Value::UInt(val_u32 as u64))
        );

        let val_u64: u64 = 42;
        batch = batch.attribute("attr.uint", val_u64);
        assert_eq!(
            batch.common.attributes.get("attr.uint"),
            Some(&Value::UInt(val_u64))
        );

        // Test Int attribute
        let val_i32: i32 = -5;
        batch.set_attribute("attr.int", val_i32);
        assert_eq!(
            batch.common.attributes.get("attr.int"),
            Some(&Value::Int(val_i32 as i64))
        );

        let val_i64: i64 = -42;
        batch = batch.attribute("attr.int", val_i64);
        assert_eq!(
            batch.common.attributes.get("attr.int"),
            Some(&Value::Int(val_i64))
        );

        // Test Float attribute
        let val_f32: f32 = 3.14;
        batch.set_attribute("attr.float", val_f32);
        assert_eq!(
            batch.common.attributes.get("attr.float"),
            Some(&Value::Float(val_f32 as f64))
        );

        let val_f64: f64 = 6.28;
        batch = batch.attribute("attr.float", val_f64);
        assert_eq!(
            batch.common.attributes.get("attr.float"),
            Some(&Value::Float(val_f64))
        );

        // Test Bool attribute
        batch.set_attribute("attr.bool", true);
        assert_eq!(
            batch.common.attributes.get("attr.bool"),
            Some(&Value::Bool(true))
        );

        batch = batch.attribute("attr.bool", false);
        assert_eq!(
            batch.common.attributes.get("attr.bool"),
            Some(&Value::Bool(false))
        );
    }

    #[test]
    fn spanbatch_from() {
        let vec = span_vec(23);
        let batch = SpanBatch::from(vec.clone());
        assert_eq!(batch.spans, vec);
        assert_eq!(batch.spans.len(), 23);
    }

    #[test]
    fn spanbatch_record() {
        let mut batch = SpanBatch::new();
        let span = Span::new("id0", "trace_id0", 9);
        batch.record(span.clone());
        assert_eq!(batch.spans.len(), 1);
        assert_eq!(batch.spans[0], span);
    }

    #[test]
    fn spanbatch_format() {
        let batch = SpanBatch::from(span_vec(23))
            .attribute("one", 1)
            .attribute("two", "too")
            .attribute("three", 3.0);

        let batch_string = format!("{}", batch);
        assert_eq!(batch_string, "<SpanBatch spans:23 attributes:3>");
    }

    #[test]
    fn spanbatch_attribute_chain() {
        let batch = SpanBatch::new()
            .attribute("bad_dogs", 0)
            .attribute("howdy", "y'all");
        let expected_string =
            r#"{"spans":[],"common":{"attributes":{"bad_dogs":0,"howdy":"y'all"}}}"#;
        assert_eq!(batch.marshall().unwrap(), expected_string);
    }
}
