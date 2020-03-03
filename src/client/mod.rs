#[cfg(not(feature = "blocking"))]
mod r#async;

#[cfg(not(feature = "blocking"))]
pub use r#async::Client;

#[cfg(feature = "blocking")]
mod sync;

#[cfg(feature = "blocking")]
pub use sync::Client;
