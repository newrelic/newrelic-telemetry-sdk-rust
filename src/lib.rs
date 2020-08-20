pub mod attribute;
#[cfg(feature = "client")]
mod client;
pub mod span;

#[cfg(feature = "client")]
pub use client::{Client, ClientBuilder};
