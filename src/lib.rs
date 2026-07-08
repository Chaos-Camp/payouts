//! # payouts
//!
//! Sends payouts to Tidepool's payment provider, and retries the ones that
//! fail for reasons worth retrying.
//!
//! The crate depends on nothing but `std`.

#![forbid(unsafe_code)]

pub mod config;
pub mod money;
pub mod provider;
pub mod request;
pub mod retry;

pub use config::RetryConfig;
pub use money::{Amount, Currency, MoneyError};
pub use provider::{FakeProvider, Provider, ProviderError, Receipt};
pub use request::{PayoutRequest, PayoutRequestBuilder, RequestError};
pub use retry::{
    backoff, send_with_config, send_with_retry, NoSleep, RetryError, Sleeper, ThreadSleeper,
};
