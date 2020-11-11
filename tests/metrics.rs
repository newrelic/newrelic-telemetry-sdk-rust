//
// Copyright 2020 New Relic Corporation. All rights reserved.
// SPDX-License-Identifier: Apache-2.0
//
#[cfg(feature = "client")]
#[macro_use]
mod common;

#[cfg(feature = "client")]
mod metrics {
    use super::common;
    use common::Endpoint;
    use anyhow::Result;
    use std::thread;
    use newrelic_telemetry::{
        Client, ClientBuilder, CountMetric, GaugeMetric, MetricBatch, SummaryMetric,
    };

    pub fn setup() -> Result<(Endpoint, Client)> {
        let _ = env_logger::builder().is_test(true).try_init();

        let endpoint = Endpoint::new();
        let client = ClientBuilder::new(&endpoint.license)
            .endpoint_metrics(&endpoint.host, Some(endpoint.port))
            .tls(false)
            .build()?;

        Ok((endpoint, client))
    }

    #[tokio::test(threaded_scheduler)]
    async fn empty() -> Result<()> {
        let (mut endpoint, client) = setup()?;

        let handle = thread::spawn(move || -> Result<()> {
            endpoint.reply(202)?;

            assert_json_eq!(
                &endpoint.next_payload().unwrap().body,
                r#"[{ "metrics" : [] }]"#
            );

            Ok(())
        });

        client.send_metrics(MetricBatch::new()).await;

        handle.join().expect("error from endpoint thread")?;

        Ok(())
    }

    #[tokio::test(threaded_scheduler)]
    async fn gauge() -> Result<()> {
        let (mut endpoint, client) = setup()?;

        let handle = thread::spawn(move || -> Result<()> {
            endpoint.reply(202)?;

            assert_json_eq!(
                r#"
        [
          {
            "metrics": [
              { 
                "name": "g1",
                "type": "gauge",
                "timestamp": 1000,
                "value": 3.14
              }
            ]
          }
        ]"#,
                &endpoint.next_payload()?.body
            );

            Ok(())
        });

        let mut metric_batch = MetricBatch::new();
        metric_batch.record(GaugeMetric::new("g1").value(3.14).timestamp(1000))?;
        client.send_metrics(metric_batch).await;

        handle.join().expect("error from endpoint thread")?;

        Ok(())
    }

    #[tokio::test(threaded_scheduler)]
    async fn count() -> Result<()> {
        let (mut endpoint, client) = setup()?;

        let handle = thread::spawn(move || -> Result<()> {
            endpoint.reply(202)?;

            assert_json_eq!(
                r#"
        [
          {
            "metrics": [
              { 
                "name": "counter",
                "type": "count",
                "timestamp": 1000,
                "interval.ms": 100,
                "value": 3.14
              }
            ]
          }
        ]"#,
                &endpoint.next_payload()?.body
            );

            Ok(())
        });

        let mut metric_batch = MetricBatch::new();
        metric_batch.record(
            CountMetric::new("counter")
                .value(3.14)
                .interval(100)
                .timestamp(1000),
        )?;

        client.send_metrics(metric_batch).await;
        handle.join().expect("error from endpoint thread")?;

        Ok(())
    }

    #[tokio::test(threaded_scheduler)]
    async fn summary() -> Result<()> {
        let (mut endpoint, client) = setup()?;

        let handle = thread::spawn(move || -> Result<()> {
            endpoint.reply(202)?;

            assert_json_eq!(
                r#"
        [
          {
            "metrics": [
              { 
                "name": "summary",
                "type": "summary",
                "timestamp": 1000,
                "interval.ms": 100,
                "value": {
                  "count": 30,
                  "sum": 3000.0,
                  "max": 100.0,
                  "min": 0.0
                }
              }
            ]
          }
        ]"#,
                &endpoint.next_payload()?.body
            );

            Ok(())
        });

        let mut metric_batch = MetricBatch::new();
        metric_batch.record(
            SummaryMetric::new("summary")
                .value(30, 3000., 0., 100.)
                .interval(100)
                .timestamp(1000),
        )?;

        client.send_metrics(metric_batch).await;
        handle.join().expect("error from endpoint thread")?;

        Ok(())
    }

    #[tokio::test(threaded_scheduler)]
    async fn batch() -> Result<()> {
        let (mut endpoint, client) = setup()?;

        let handle = thread::spawn(move || -> Result<()> {
            endpoint.reply(202)?;

            assert_json_eq!(
                r#"
        [
          {
            "common" : {
              "attributes" : {
                "host": "hostname",
                "priority": 3.14
              }
            },
            "metrics": [
              { 
                "name": "g1",
                "type": "gauge",
                "timestamp": 1000,
                "value": 3.14
              },
              { 
                "name": "counter",
                "type": "count",
                "timestamp": 1000,
                "interval.ms": 100,
                "value": 3.14
              },
              { 
                "name": "summary",
                "type": "summary",
                "timestamp": 1000,
                "interval.ms": 100,
                "value": {
                  "count": 30,
                  "sum": 3000.0,
                  "max": 100.0,
                  "min": 0.0
                }
              }
            ]
          }
        ]"#,
                &endpoint.next_payload()?.body
            );

            Ok(())
        });

        let mut metric_batch = MetricBatch::new();
        metric_batch.record(GaugeMetric::new("g1").value(3.14).timestamp(1000))?;
        metric_batch.record(
            CountMetric::new("counter")
                .value(3.14)
                .interval(100)
                .timestamp(1000),
        )?;
        metric_batch.record(
            SummaryMetric::new("summary")
                .value(30, 3000., 0., 100.)
                .interval(100)
                .timestamp(1000),
        )?;

        metric_batch.add_attribute("host", "hostname");
        metric_batch.add_attribute("priority", 3.14);

        client.send_metrics(metric_batch).await;
        handle.join().expect("error from endpoint thread")?;

        Ok(())
    }
}
