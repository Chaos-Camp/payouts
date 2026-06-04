//! # payouts
//!
//! Sends payouts to Tidepool's payment provider, and retries the ones that
//! fail for reasons worth retrying.
//!
//! The crate depends on nothing but `std`.

#![forbid(unsafe_code)]

pub mod money;

pub use money::{Amount, Currency};
