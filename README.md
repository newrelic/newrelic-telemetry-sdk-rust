[![Community Project header](https://github.com/newrelic/open-source-office/raw/master/examples/categories/images/Community_Project.png)](https://github.com/newrelic/open-source-office/blob/master/examples/categories/index.md#community-project)

# New Relic Rust Telemetry SDK

![Build status](https://github.com/newrelic/newrelic-telemetry-sdk-rust/workflows/CI/badge.svg)
[![Release](https://img.shields.io/github/v/release/newrelic/newrelic-telemetry-sdk-rust?include_prereleases&style=)](https://github.com/newrelic/newrelic-telemetry-sdk-rust/releases/)

What is the New Relic Rust Telemetry SDK?

* It's a helper library that supports sending New Relic data from within your Rust application.
* It's the foundation of [New Relic's C Telemetry SDK](https://github.com/newrelic/newrelic-telemetry-sdk-c).
* It’s an example of "best practices" for sending us data.

This SDK currently supports sending spans to the [Trace API](https://docs.newrelic.com/docs/understand-dependencies/distributed-tracing/trace-api/introduction-trace-api).

**This project is currently in an alpha state.**

## Getting Started

This is a simple application that sends a single span via the asynchronous
client:
```rust
use newrelic_telemetry::{ClientBuilder, Span};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() {
    // Obtain a license key from the environment.
    let license_key = env::var("NEW_RELIC_API_KEY").unwrap();

    // Build an asynchronous client.
    let client = ClientBuilder::new(&license_key).build().unwrap();

    // Create a span and a span batch.
    let span = Span::new(
        "e9f54a2c322d7578",
        "1b1bf29379951c1d",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
    )
    .name("/index.html")
    .attribute("username", "user")
    .service_name("Telemetry Application");

    let span_batch = vec![span].into();

    // Send a span batch and await the future.
    client.send_spans(span_batch).await;
}
```

## Find and use your data

Tips on how to find and query your data in New Relic:
- [Find metric data](https://docs.newrelic.com/docs/data-ingest-apis/get-data-new-relic/metric-api/introduction-metric-api#find-data)
- [Find trace/span data](https://docs.newrelic.com/docs/understand-dependencies/distributed-tracing/trace-api/introduction-trace-api#view-data)

For general querying information, see:
- [Query New Relic data](https://docs.newrelic.com/docs/using-new-relic/data/understand-data/query-new-relic-data)
- [Intro to NRQL](https://docs.newrelic.com/docs/query-data/nrql-new-relic-query-language/getting-started/introduction-nrql)

## Contributing

We encourage your contributions to improve the Rust Telemetry SDK! Keep in mind
when you submit your pull request, you'll need to sign the CLA via the 
click-through using CLA-Assistant. You only have to sign the CLA one time per
project. If you have any questions, or to execute our corporate CLA, required
if your contribution is on behalf of a company,  please drop us an email at
opensource@newrelic.com.

## License

The Rust Telemetry SDK is licensed under the [Apache 2.0](http://apache.org/licenses/LICENSE-2.0.txt) 
License.

### Limitations

The New Relic Telemetry APIs are rate limited. Please reference the
documentation for [New Relic Metric API](https://docs.newrelic.com/docs/introduction-new-relic-metric-api) 
and [New Relic Trace API requirements and limits](https://docs.newrelic.com/docs/apm/distributed-tracing/trace-api/trace-api-general-requirements-limits)
on the specifics of the rate limits.
