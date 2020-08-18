use anyhow::{anyhow, Result};
use std::time::Duration;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

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

/// `ClientBuilder` acts as builder for initializing a `Client`.
///
/// It can be used to customize ingest URLs, the backoff factor, the retry
/// maximum, and the product info.
///
/// ```
/// # use newrelic_telemetry::ClientBuilder;
/// # use std::time::Duration;
/// # let api_key = "";
/// let mut builder = ClientBuilder::new(api_key);
///
/// let client = builder.backoff_factor(Duration::from_secs(10))
///                     .product_info("RustDoc", "1.0")
///                     .build();
/// ```
pub struct ClientBuilder {
    api_key: String,
    backoff_factor: Duration,
    retries_max: u32,
    endpoint_traces: (String, u32),
    product_info: Option<(String, String)>,
}

impl ClientBuilder {
    /// Initialize the client builder with an API key.
    ///
    /// Other values will be set to defaults:
    ///  * The default backoff factor will be 5 seconds.
    ///  * The default maximum of retries is 8.
    ///  * The default trace endpoint is `https://trace-api.newrelic.com/trace/v1` on port 80.
    ///  * By default, product information is empty.
    ///
    /// ```
    /// # use newrelic_telemetry::ClientBuilder;
    /// # let api_key = "";
    /// let mut builder = ClientBuilder::new(api_key);
    /// ```
    pub fn new(api_key: &str) -> Self {
        ClientBuilder {
            api_key: api_key.to_string(),
            backoff_factor: Duration::from_secs(5),
            retries_max: 8,
            endpoint_traces: ("https://trace-api.newrelic.com/trace/v1".to_string(), 80),
            product_info: None,
        }
    }

    /// Configures a backoff factor.
    ///
    /// If a request fails, the SDK retries the request at increasing intervals
    /// and eventually drops data if the request cannot be completed.
    ///
    /// The amount of time to wait after a request can be computed using this
    /// logic:
    ///
    ///   `backoff_factor * (2 ^ (number_of_retries - 1))`
    ///
    /// For a backoff factor of 1 second, and a maximum of 6 retries, the retry
    /// delay interval should follow a pattern of [0, 1, 2, 4, 8, 16].
    ///
    /// See the [specification](https://github.com/newrelic/newrelic-telemetry-sdk-specs/blob/master/communication.md#graceful-degradation)
    /// for further details.
    ///
    /// ```
    /// # use newrelic_telemetry::ClientBuilder;
    /// # use std::time::Duration;
    /// # let api_key = "";
    /// let mut builder =
    ///     ClientBuilder::new(api_key).backoff_factor(Duration::from_secs(10));
    /// ```
    pub fn backoff_factor(mut self, factor: Duration) -> Self {
        self.backoff_factor = factor;
        self
    }

    /// Configures the maximum numbers of retries.
    ///
    /// If a request fails, the SDK retries the request at increasing intervals
    /// and eventually drops data if the request cannot be completed.
    ///
    /// If zero is given as a maximum, no retries will be made for failed
    /// requests.
    ///
    /// For a backoff factor of 1 second, and a maximum of 6 retries, the retry
    /// delay interval should follow a pattern of [0, 1, 2, 4, 8, 16].
    ///
    /// See the [specification](https://github.com/newrelic/newrelic-telemetry-sdk-specs/blob/master/communication.md#graceful-degradation)
    /// for further details.
    ///
    /// ```
    /// # use newrelic_telemetry::ClientBuilder;
    /// # let api_key = "";
    /// let mut builder =
    ///     ClientBuilder::new(api_key).retries_max(4);
    /// ```
    pub fn retries_max(mut self, retries: u32) -> Self {
        self.retries_max = retries;
        self
    }

    /// Configure the ingest URL for traces.
    ///
    /// Overrides the default ingest URL for traces to facilitate communication
    /// with alternative New Relic backends.
    ///
    /// ```
    /// # use newrelic_telemetry::ClientBuilder;
    /// # let api_key = "";
    /// let mut builder =
    ///     ClientBuilder::new(api_key).endpoint_traces("https://127.0.0.1/trace/v1", 80);
    /// ```
    pub fn endpoint_traces(mut self, url: &str, port: u32) -> Self {
        self.endpoint_traces = (url.to_string(), port);
        self
    }

    /// Configure a product and version.
    ///
    /// The specified product and version will be appended to the `User-Agent`
    /// header of payloads.
    ///
    /// See the [specification](https://github.com/newrelic/newrelic-telemetry-sdk-specs/blob/master/communication.md#user-agent)
    /// for further details.
    ///
    /// ```
    /// # use newrelic_telemetry::ClientBuilder;
    /// # let api_key = "";
    /// let mut builder =
    ///     ClientBuilder::new(api_key).product_info("NewRelic-Cpp-OpenTelemetry", "0.2.1");
    /// ```
    pub fn product_info(mut self, product: &str, version: &str) -> Self {
        self.product_info = Some((product.to_string(), version.to_string()));
        self
    }

    /// Build an asynchronous client.
    ///
    /// ```
    /// # use newrelic_telemetry::ClientBuilder;
    /// # let api_key = "";
    /// let builder = ClientBuilder::new(api_key);
    ///
    /// let client = builder.build();
    /// ```
    pub fn build(self) -> Result<r#async::Client> {
        Err(anyhow!("not implemented"))
    }

    fn get_backoff_sequence(&self) -> Vec<Duration> {
        (0..self.retries_max)
            .map(|num_retry| {
                if num_retry == 0 {
                    Duration::from_secs(0)
                } else {
                    self.backoff_factor * (2_u32.pow(num_retry - 1))
                }
            })
            .collect()
    }

    fn get_user_agent_header(&self) -> String {
        let product_info = match &self.product_info {
            Some(s) => format!(" {}/{}", s.0, s.1),
            _ => "".to_string(),
        };

        format!("NewRelic-Rust-TelemetrySDK/{}{}", VERSION, product_info)
    }
}

/// An async implementation of the New Relic Telemetry SDK `Client`.
pub mod r#async {
    use super::Sendable;
    use anyhow::{anyhow, Result};
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use hyper::{HeaderMap, Response};
    use log::{debug, error, info};
    use std::io::Write;
    use std::time::Duration;

    // An internal enum representing the state of a payload.
    #[derive(Debug, PartialEq)]
    enum SendableState {
        // No retry should be made.
        Done,

        // A retry should be made. Either after the given duration, or, if it
        // is `None`, according to the backoff sequence.
        Retry(Option<Duration>),

        // The payload should be split and a retry should be made for both
        // payloads.
        Split,
    }

    pub struct Client {}

    impl Client {
        // Returns a gzip compressed version of the given string.
        fn to_gzip(text: &String) -> Result<Vec<u8>> {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(text.as_bytes())?;
            Ok(encoder.finish()?)
        }

        // Extract the value of the Retry-After HTTP response header
        fn extract_retry_after(headers: &HeaderMap) -> Result<Duration> {
            if let Some(dur) = headers.get("retry-after") {
                Ok(Duration::from_secs(dur.to_str()?.parse::<u64>()?))
            } else {
                Err(anyhow!("missing retry-after header"))
            }
        }

        // Based on the response from an ingest endpoint, decide whether to
        // retry or split a payload.
        //
        // See the [specification](https://github.com/newrelic/newrelic-telemetry-sdk-specs/blob/master/communication.md#response-codes)
        // for further details.
        fn process_response<T>(batch: Box<dyn Sendable>, response: Response<T>) -> SendableState {
            let status = response.status();

            match status.as_u16() {
                200..=299 => {
                    debug!("response {}, successfully sent {}", status, batch);
                }
                400 | 401 | 403 | 404 | 405 | 409 | 410 | 411 => {
                    error!("response {}, dropping {}", status, batch);
                }
                413 => {
                    info!(
                        "response {}, payload too large, splitting {}",
                        status, batch
                    );
                    return SendableState::Split;
                }
                429 => match Self::extract_retry_after(response.headers()) {
                    Ok(duration) => {
                        info!(
                            "response {}: retry interval {:?}, retrying {}",
                            status, duration, batch
                        );

                        return SendableState::Retry(Some(duration));
                    }
                    Err(e) => {
                        error!("response {}, {}, dropping {}", status, e, batch);
                    }
                },
                _ => {
                    debug!("response {}, retry {}", status, batch);
                    return SendableState::Retry(None);
                }
            }
            return SendableState::Done;
        }
    }

    #[cfg(test)]
    mod tests {
        use super::{Client, Sendable, SendableState};
        use anyhow::Result;
        use flate2::read::GzDecoder;
        use hyper::Response;
        use std::fmt;
        use std::io::Read;
        use std::time::Duration;

        pub struct TestBatch;

        impl Sendable for TestBatch {
            fn marshall(&self) -> Result<String> {
                Ok("".to_string())
            }

            fn split(&mut self) -> Box<dyn Sendable> {
                Box::new(TestBatch)
            }
        }

        impl fmt::Display for TestBatch {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "<TestBatch>")
            }
        }

        #[test]
        fn to_gzip() -> Result<()> {
            let text = "Text to be encoded".to_string();
            let encoded = Client::to_gzip(&text)?;

            let mut gz = GzDecoder::new(&encoded[..]);
            let mut decoded = String::new();
            gz.read_to_string(&mut decoded)?;

            assert_eq!(decoded, text);

            Ok(())
        }

        #[test]
        fn extract_retry_after() -> Result<()> {
            let mut headers = hyper::HeaderMap::new();

            let when = Client::extract_retry_after(&headers);
            assert!(when.is_err());

            headers.insert("Retry-after", "7".parse()?);

            let when = Client::extract_retry_after(&headers)?;
            assert_eq!(when, Duration::from_secs(7));

            headers.insert("Retry-after", "seven".parse()?);

            let when = Client::extract_retry_after(&headers);
            assert!(when.is_err());

            Ok(())
        }

        #[test]
        fn process_response_success() -> Result<()> {
            for code in 200..300 {
                let batch = Box::new(TestBatch);
                let response = Response::builder().status(code).body(())?;

                assert_eq!(
                    Client::process_response(batch, response),
                    SendableState::Done
                );
            }

            Ok(())
        }

        #[test]
        fn process_response_error() -> Result<()> {
            for code in vec![400, 401, 403, 404, 405, 409, 410, 411] {
                let batch = Box::new(TestBatch);
                let response = Response::builder().status(code).body(())?;

                assert_eq!(
                    Client::process_response(batch, response),
                    SendableState::Done
                );
            }

            Ok(())
        }

        #[test]
        fn process_response_split() -> Result<()> {
            let batch = Box::new(TestBatch);
            let response = Response::builder().status(413).body(())?;

            assert_eq!(
                Client::process_response(batch, response),
                SendableState::Split
            );

            Ok(())
        }

        #[test]
        fn process_response_retry_from_header() -> Result<()> {
            let batch = Box::new(TestBatch);
            let response = Response::builder()
                .status(429)
                .header("retry-after", "7")
                .body(())?;

            assert_eq!(
                Client::process_response(batch, response),
                SendableState::Retry(Some(Duration::from_secs(7)))
            );

            Ok(())
        }

        #[test]
        fn process_response_retry() -> Result<()> {
            let mut codes = vec![402, 406, 407, 408];
            codes.append(&mut (100..200).collect());
            codes.append(&mut (300..400).collect());
            codes.append(&mut (430..600).collect());

            for code in codes {
                let batch = Box::new(TestBatch);
                let response = Response::builder().status(code).body(())?;

                assert_eq!(
                    Client::process_response(batch, response),
                    SendableState::Retry(None),
                    "expected retry on {}",
                    code
                );
            }

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ClientBuilder, VERSION};
    use std::time::Duration;

    #[test]
    fn builder_default() {
        let b = ClientBuilder::new("0000");

        assert_eq!(b.api_key, "0000");
        assert_eq!(b.backoff_factor, Duration::from_secs(5));
        assert_eq!(b.retries_max, 8);
        assert_eq!(
            b.endpoint_traces,
            ("https://trace-api.newrelic.com/trace/v1".to_string(), 80)
        );
        assert_eq!(b.product_info, None);
    }

    #[test]
    fn builder_setters() {
        let b = ClientBuilder::new("0000")
            .backoff_factor(Duration::from_secs(10))
            .retries_max(10)
            .endpoint_traces("https://127.0.0.1", 8080)
            .product_info("Test", "1.0");

        assert_eq!(b.api_key, "0000");
        assert_eq!(b.backoff_factor, Duration::from_secs(10));
        assert_eq!(b.retries_max, 10);
        assert_eq!(b.endpoint_traces, ("https://127.0.0.1".to_string(), 8080));
        assert_eq!(
            b.product_info,
            Some(("Test".to_string(), "1.0".to_string()))
        );
    }

    #[test]
    fn backoff_sequence_default() {
        let seq = ClientBuilder::new("").get_backoff_sequence();

        assert_eq!(
            seq,
            vec![0, 5, 10, 20, 40, 80, 160, 320]
                .into_iter()
                .map(|d| Duration::from_secs(d))
                .collect::<Vec<Duration>>()
        );
    }

    #[test]
    fn backoff_sequence_no_retry() {
        let seq = ClientBuilder::new("").retries_max(0).get_backoff_sequence();

        assert_eq!(seq, vec![]);
    }

    #[test]
    fn backoff_sequence_custom() {
        let seq = ClientBuilder::new("")
            .backoff_factor(Duration::from_secs(2))
            .retries_max(6)
            .get_backoff_sequence();

        assert_eq!(
            seq,
            vec![0, 2, 4, 8, 16, 32]
                .into_iter()
                .map(|d| Duration::from_secs(d))
                .collect::<Vec<Duration>>()
        );
    }

    #[test]
    fn user_agent_header_default() {
        let header = ClientBuilder::new("").get_user_agent_header();

        assert_eq!(header, format!("NewRelic-Rust-TelemetrySDK/{}", VERSION));
    }

    #[test]
    fn user_agent_header_custom() {
        let header = ClientBuilder::new("")
            .product_info("Doc", "1.0")
            .get_user_agent_header();

        assert_eq!(
            header,
            format!("NewRelic-Rust-TelemetrySDK/{} Doc/1.0", VERSION)
        );
    }
}
