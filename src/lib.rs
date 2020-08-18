pub mod attribute;
pub mod span;
mod client;

#[cfg(feature = "client")]
pub use client::r#async;
#[cfg(feature = "client")]
pub use client::ClientBuilder;
