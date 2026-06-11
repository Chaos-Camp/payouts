//! The payment provider boundary.
//!
//! Everything on the far side of [`Provider::send`] is someone else's
//! network, someone else's database and someone else's on-call rota. The
//! only thing this crate needs to know about a failure is whether trying
//! again might help, which is what [`ProviderError`] encodes.

use std::cell::Cell;
use std::fmt;

use crate::request::PayoutRequest;

/// Proof that the provider accepted a payout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Receipt {
    /// The provider's identifier for the payout, e.g. `"po_0001"`.
    pub id: String,
    /// Our reference, echoed back so receipts can be reconciled.
    pub reference: String,
}

/// Why a provider rejected a payout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderError {
    /// Worth retrying: the provider answered with this HTTP status
    /// (typically 429, 502, 503 or 504) and may succeed next time.
    Transient(u16),
    /// Not worth retrying: the account is closed, the recipient is unknown,
    /// the request is malformed. Asking again would get the same answer.
    Fatal(String),
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProviderError::Transient(status) => write!(f, "provider returned HTTP {status}"),
            ProviderError::Fatal(reason) => write!(f, "provider rejected payout: {reason}"),
        }
    }
}

impl std::error::Error for ProviderError {}

/// Something that can move money to a recipient.
///
/// Implementations talk to a real provider over the network. They should
/// map rate limiting and 5xx responses to [`ProviderError::Transient`] and
/// everything else to [`ProviderError::Fatal`]. Retrying is the caller's
/// job; see [`crate::send_with_retry`].
pub trait Provider {
    /// Sends one payout, once.
    fn send(&self, req: &PayoutRequest) -> Result<Receipt, ProviderError>;
}

/// A scripted provider for tests: fails a set number of times, then succeeds.
#[derive(Debug)]
pub struct FakeProvider {
    failures_left: Cell<u32>,
    failure: ProviderError,
    calls: Cell<u32>,
}

impl FakeProvider {
    /// Accepts every payout.
    pub fn succeeding() -> Self {
        Self::failing(0, 503)
    }

    /// Fails the first `times` calls with `Transient(status)`, then succeeds.
    pub fn failing(times: u32, status: u16) -> Self {
        FakeProvider {
            failures_left: Cell::new(times),
            failure: ProviderError::Transient(status),
            calls: Cell::new(0),
        }
    }

    /// Fails every call with `Transient(status)`. (Every call for the next
    /// four billion, which is long enough for a test.)
    pub fn always_failing(status: u16) -> Self {
        Self::failing(u32::MAX, status)
    }

    /// How many times `send` has been called.
    pub fn calls(&self) -> u32 {
        self.calls.get()
    }
}

impl Provider for FakeProvider {
    fn send(&self, req: &PayoutRequest) -> Result<Receipt, ProviderError> {
        let call = self.calls.get() + 1;
        self.calls.set(call);

        let failures_left = self.failures_left.get();
        if failures_left > 0 {
            self.failures_left.set(failures_left - 1);
            return Err(self.failure.clone());
        }

        Ok(Receipt {
            id: format!("po_{call:04}"),
            reference: req.reference().to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::money::{Amount, Currency};

    fn request(amount: Amount) -> PayoutRequest {
        PayoutRequest::builder()
            .recipient("acct_123")
            .amount(amount)
            .reference("INV-1001")
            .build()
            .unwrap()
    }

    #[test]
    fn fake_provider_fails_then_succeeds() {
        let provider = FakeProvider::failing(2, 503);
        let req = request(Amount::new(1250, Currency::Usd));
        assert_eq!(provider.send(&req), Err(ProviderError::Transient(503)));
        assert_eq!(provider.send(&req), Err(ProviderError::Transient(503)));
        assert_eq!(provider.send(&req).unwrap().id, "po_0003");
        assert_eq!(provider.calls(), 3);
    }
}
