use anyhow::Result;

/// Types that can be sent to a New Relic ingest API
///
/// New Relic ingest APIs currently accept batches of traces, metrics, events
/// or logs.
pub trait Sendable: std::fmt::Display + Send {
    // Create a payload
    //
    // This method creates a JSON payload representing the contents of the
    // `Sendable` object, conforming to the requirements of a related ingest
    // API (traces, metrics, events or logs).
    fn marshall(&self) -> Result<String>;

    // Split a `Sendable`
    //
    // New Relic ingest APIs reject payloads that are too large. In that case,
    // a 413 response code is sent, the payload must be split and sent again
    // (see [the specification](https://github.com/newrelic/newrelic-telemetry-sdk-specs/blob/master/communication.md#response-codes)
    // for further details).
    //
    // This method removes half of the content of the `Sendable` object and
    // puts it into a second `Sendable` object, which is returned.
    fn split(&mut self) -> Box<dyn Sendable>;
}
