//! # payouts
//!
//! Sends payouts to Tidepool's payment provider, and retries the ones that
//! fail for reasons worth retrying.
//!
//! ```
//! use payouts::{send_with_config, Amount, FakeProvider, NoSleep, PayoutRequest, RetryConfig};
//!
//! let req = PayoutRequest::builder()
//!     .recipient("acct_8812")
//!     .amount("12.50 USD".parse::<Amount>()?)
//!     .reference("INV-1001")
//!     .build()?;
//!
//! // A provider that answers 503 twice, then accepts the payout.
//! let provider = FakeProvider::failing(2, 503);
//! let receipt = send_with_config(&provider, &req, &RetryConfig::default(), &mut NoSleep)?;
//! assert_eq!(receipt.reference, "INV-1001");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! In production, pass a real [`Provider`] and a [`ThreadSleeper`].
//!
//! The crate depends on nothing but `std`.

#![forbid(unsafe_code)]
#![warn(missing_debug_implementations)]

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
