//! The payment provider boundary.
//!
//! Everything on the far side of [`Provider::send`] is someone else's
//! network, someone else's database and someone else's on-call rota. The
//! only thing this crate needs to know about a failure is whether trying
//! again might help, which is what [`ProviderError`] encodes.

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
