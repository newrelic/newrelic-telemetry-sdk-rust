//
// Copyright 2020 New Relic Corporation. All rights reserved.
// SPDX-License-Identifier: Apache-2.0
//
use crate::attribute::Value;
use crate::client::Sendable;
use anyhow::{anyhow, Result};
use log::error;
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt;
use std::time::SystemTime;
use uuid::Uuid;

pub fn now_as_millis() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_millis()
        .try_into()?)
}

pub trait Metric: Send + fmt::Debug {
    fn valid(&mut self) -> Result<()>;
    fn json(&self) -> Result<serde_json::Value>;
}

/// Represents a gauge metric
#[derive(Serialize, Debug, PartialEq)]
pub struct GaugeMetric {
    name: String,

    #[serde(rename = "type")]
    typename: &'static str,

    value: Option<f64>,

    timestamp: Option<u64>,

    #[serde(skip_serializing_if = "HashMap::is_empty")]
    attributes: HashMap<String, Value>,
}

/// Represents a count metric
#[derive(Serialize, Debug, PartialEq)]
pub struct CountMetric {
    name: String,

    #[serde(rename = "type")]
    typename: &'static str,

    value: Option<f64>,

    timestamp: Option<u64>,

    #[serde(rename = "interval.ms")]
    interval: Option<u64>,

    #[serde(skip_serializing_if = "HashMap::is_empty")]
    attributes: HashMap<String, Value>,
}

#[derive(Serialize, Debug, PartialEq)]
struct SummaryValue {
    count: u64,
    sum: f64,
    min: f64,
    max: f64,
}

/// Represents a summary metric
#[derive(Serialize, Debug, PartialEq)]
pub struct SummaryMetric {
    name: String,

    #[serde(rename = "type")]
    typename: &'static str,

    value: Option<SummaryValue>,

    timestamp: Option<u64>,

    #[serde(rename = "interval.ms")]
    interval: Option<u64>,

    #[serde(skip_serializing_if = "HashMap::is_empty")]
    attributes: HashMap<String, Value>,
}

impl Metric for GaugeMetric {
    fn json(&self) -> Result<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }

    fn valid(&mut self) -> Result<()> {
        if self.timestamp == None {
            self.timestamp = Some(now_as_millis().unwrap_or(0));
        }

        if self.value == None {
            Err(anyhow!("metric requires a value"))
        } else {
            Ok(())
        }
    }
}

impl Metric for CountMetric {
    fn json(&self) -> Result<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }

    fn valid(&mut self) -> Result<()> {
        if self.timestamp == None {
            self.timestamp = Some(now_as_millis().unwrap_or(0));
        }

        if self.value == None {
            return Err(anyhow!("metric requires a value"));
        }

        if self.interval == None {
            return Err(anyhow!("metric requires a value"));
        }

        Ok(())
    }
}

impl Metric for SummaryMetric {
    fn json(&self) -> Result<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }

    fn valid(&mut self) -> Result<()> {
        if self.timestamp == None {
            self.timestamp = Some(now_as_millis().unwrap_or(0));
        }

        if self.value == None {
            return Err(anyhow!("metric requires a value"));
        }

        if self.interval == None {
            return Err(anyhow!("metric requires a value"));
        }

        Ok(())
    }
}

impl GaugeMetric {
    pub fn new(name: &str) -> GaugeMetric {
        GaugeMetric {
            name: name.to_string(),
            typename: "gauge",
            value: None,
            timestamp: None,
            attributes: HashMap::new(),
        }
    }

    pub fn value(mut self, value: f64) -> Self {
        self.value = Some(value);
        self
    }

    pub fn timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// Set an attribute on the gauge metric.
    pub fn attribute<T: Into<Value>>(mut self, key: &str, value: T) -> Self {
        self.attributes.insert(key.to_string(), value.into());
        self
    }
}

impl CountMetric {
    pub fn new(name: &str) -> CountMetric {
        CountMetric {
            name: name.to_string(),
            typename: "count",
            value: None,
            timestamp: None,
            interval: None,
            attributes: HashMap::new(),
        }
    }

    pub fn value(mut self, value: f64) -> Self {
        self.value = Some(value);
        self
    }

    pub fn timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    pub fn interval(mut self, interval: u64) -> Self {
        self.interval = Some(interval);
        self
    }

    /// Set an attribute on the count metric.
    pub fn attribute<T: Into<Value>>(mut self, key: &str, value: T) -> Self {
        self.attributes.insert(key.to_string(), value.into());
        self
    }
}

impl SummaryMetric {
    pub fn new(name: &str) -> SummaryMetric {
        SummaryMetric {
            name: name.to_string(),
            typename: "summary",
            value: None,
            timestamp: None,
            interval: None,
            attributes: HashMap::new(),
        }
    }

    pub fn value(mut self, count: u64, sum: f64, min: f64, max: f64) -> Self {
        self.value = Some(SummaryValue {
            count,
            sum,
            min,
            max,
        });
        self
    }

    pub fn timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    pub fn interval(mut self, interval: u64) -> Self {
        self.interval = Some(interval);
        self
    }

    /// Set an attribute on the summary metric.
    pub fn attribute<T: Into<Value>>(mut self, key: &str, value: T) -> Self {
        self.attributes.insert(key.to_string(), value.into());
        self
    }
}

pub struct MetricBatch {
    uuid: String,

    metrics: Vec<Box<dyn Metric>>,
    attributes: HashMap<String, Value>,
}

impl MetricBatch {
    /// Create a new metric batch.
    pub fn new() -> Self {
        MetricBatch {
            uuid: Uuid::new_v4().to_string(),
            metrics: vec![],
            attributes: HashMap::new(),
        }
    }

    /// Add a common attribute for all metrics in this batch.
    pub fn add_attribute<T: Into<Value>>(&mut self, key: &str, value: T) {
        self.attributes.insert(key.to_string(), value.into());
    }

    /// Record and add the metric to the batch from which it was created.
    pub fn record<T: Metric + 'static>(&mut self, mut metric: T) -> Result<()> {
        metric.valid()?;

        self.metrics.push(Box::new(metric));

        Ok(())
    }
}

impl fmt::Display for MetricBatch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<MetricBatch, {} data points>", self.metrics.len())
    }
}

impl Sendable for MetricBatch {
    fn uuid(&self) -> &str {
        &self.uuid
    }

    fn marshall(&self) -> Result<String> {
        let mut json_metrics = vec![];

        for m in self.metrics.iter() {
            match m.json() {
                Ok(j) => json_metrics.push(j),
                Err(e) => error!("cannot convert metric {:?} to json: {}", m, e),
            }
        }

        let metrics = serde_json::to_value(json_metrics)?;
        let mut data = json!([{ "metrics": metrics }]);

        if self.attributes.len() > 0 {
            let attrs = serde_json::to_value(&self.attributes)?;
            data[0]["common"] = json!({ "attributes": attrs });
        }

        Ok(data.to_string())
    }

    fn split(&mut self) -> Box<dyn Sendable> {
        let new_batch_size: usize = self.metrics.len() / 2;
        self.uuid = Uuid::new_v4().to_string();

        Box::new(MetricBatch {
            uuid: Uuid::new_v4().to_string(),
            metrics: self.metrics.drain(new_batch_size..).collect(),
            attributes: self.attributes.clone(),
        })
    }
}
