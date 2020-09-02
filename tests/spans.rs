#[cfg(feature = "client")]
#[macro_use]
mod common;

#[cfg(feature = "client")]
mod client {
    use super::common;
    use anyhow::Result;
    use common::Endpoint;
    use std::time::Duration;
    use std::thread;
    use newrelic_telemetry::{Client, ClientBuilder, Span, SpanBatch};

    pub fn setup() -> (Endpoint, Client) {
        let _ = env_logger::builder().is_test(true).try_init();

        let endpoint = Endpoint::new();
        let client = ClientBuilder::new(&endpoint.license)
            .endpoint_traces(&endpoint.host, Some(endpoint.port))
            .tls(false)
            .build()
            .unwrap();

        (endpoint, client)
    }

    #[tokio::test(threaded_scheduler)]
    async fn empty() -> Result<()> {
        let (mut endpoint, client) = setup();

        // Assertions are all handled in a separate thread, so we can await the
        // future in the main thread.
        let handle = thread::spawn(move || -> Result<()> {
            endpoint.reply(202).unwrap();

            assert_json_eq!(
                &endpoint.next_payload().unwrap().body,
                r#"[{ "spans" : [] }]"#
            );

            Ok(())
        });

        client.send_spans(SpanBatch::new()).await;

        handle.join().unwrap()?;

        Ok(())
    }

    #[tokio::test(threaded_scheduler)]
    async fn simple() -> Result<()> {
        let (mut endpoint, client) = setup();

        let handle = thread::spawn(move || -> Result<()> {
            endpoint.reply(202)?;

            assert_json_eq!(
                &endpoint.next_payload()?.body,
                r#"
                [
                  {
                    "spans": [
                      {
                        "id": "id1",
                        "timestamp": 1000,
                        "trace.id": "tid1"
                      }
                    ]
                  }
                ]"#
            );

            Ok(())
        });

        let mut span_batch = SpanBatch::new();
        span_batch.record(Span::new("id1", "tid1", 1000));

        client.send_spans(span_batch).await;
        handle.join().unwrap()?;

        Ok(())
    }

    #[tokio::test(threaded_scheduler)]
    async fn all_api_attrs() -> Result<()> {
        let (mut endpoint, client) = setup();

        let handle = thread::spawn(move || -> Result<()> {
            endpoint.reply(202)?;

            assert_json_eq!(
                &endpoint.next_payload()?.body,
                r#"
                [{
                  "spans": [{
                    "id": "id1",
                    "timestamp": 1000,
                    "trace.id": "tid1",
                    "attributes": {
                      "name": "name1",
                      "duration.ms": 2000,
                      "parent.id": "pid1",
                      "service.name": "service1"
                    }
                  }]
                }]"#
            );

            Ok(())
        });

        let mut span_batch = SpanBatch::new();
        span_batch.record(
            Span::new("id1", "tid1", 1000)
                .name("name1")
                .duration(Duration::from_millis(2000))
                .parent_id("pid1")
                .service_name("service1"),
        );

        client.send_spans(span_batch).await;
        handle.join().unwrap()?;

        Ok(())
    }

    #[tokio::test(threaded_scheduler)]
    async fn custom_attrs() -> Result<()> {
        let (mut endpoint, client) = setup();

        let handle = thread::spawn(move || -> Result<()> {
            endpoint.reply(202)?;

            assert_json_eq!(
                &endpoint.next_payload()?.body,
                r#"
                [{
                  "spans": [{
                    "id": "id1",
                    "timestamp": 1000,
                    "trace.id": "tid1",
                    "attributes": {
                      "bool_attr": true,
                      "float_attr": 3.14159,
                      "str_attr": "string",
                      "int_attr": 40,
                      "neg_int_attr": -40,
                      "string_attr": "Str"
                    }
                  }]
                }]"#
            );

            Ok(())
        });

        let mut span_batch = SpanBatch::new();
        span_batch.record(
            Span::new("id1", "tid1", 1000)
                .attribute("bool_attr", true)
                .attribute("float_attr", 3.14159)
                .attribute("str_attr", "string")
                .attribute("int_attr", 40)
                .attribute("neg_int_attr", -40)
                .attribute("string_attr", "Str"),
        );

        client.send_spans(span_batch).await;
        handle.join().unwrap()?;

        Ok(())
    }

    #[tokio::test(threaded_scheduler)]
    async fn two_spans() -> Result<()> {
        let (mut endpoint, client) = setup();

        let handle = thread::spawn(move || -> Result<()> {
            endpoint.reply(202)?;

            assert_json_eq!(
                &endpoint.next_payload()?.body,
                r#"
                [{
                  "spans": [
                    {
                      "id": "id1",
                      "timestamp": 1000,
                      "trace.id": "tid1"
                    },
                    {
                      "id": "id2",
                      "timestamp": 2000,
                      "trace.id": "tid2"
                    }
                  ]
                }]"#
            );

            Ok(())
        });

        let mut span_batch = SpanBatch::new();
        span_batch.record(Span::new("id1", "tid1", 1000));
        span_batch.record(Span::new("id2", "tid2", 2000));

        client.send_spans(span_batch).await;
        handle.join().unwrap()?;

        Ok(())
    }
}
