//! Building a payout request.

use std::fmt;

use crate::money::Amount;

/// One payout: who gets paid, how much, and our reference for it.
///
/// Built with [`PayoutRequest::builder`], which checks every field before a
/// request can reach a provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayoutRequest {
    recipient: String,
    amount: Amount,
    reference: String,
    idempotency_key: String,
}

impl PayoutRequest {
    pub fn builder() -> PayoutRequestBuilder {
        PayoutRequestBuilder::default()
    }

    /// The provider's identifier for the recipient's account.
    pub fn recipient(&self) -> &str {
        &self.recipient
    }

    pub fn amount(&self) -> Amount {
        self.amount
    }

    /// Our reference, e.g. an invoice number. Echoed back on the receipt.
    pub fn reference(&self) -> &str {
        &self.reference
    }

    /// The key the provider deduplicates on. Every attempt at the same
    /// payout must send the same key, or a retry can pay twice.
    pub fn idempotency_key(&self) -> &str {
        &self.idempotency_key
    }
}

/// Builder for [`PayoutRequest`].
#[derive(Debug, Default, Clone)]
pub struct PayoutRequestBuilder {
    recipient: Option<String>,
    amount: Option<Amount>,
    reference: Option<String>,
    idempotency_key: Option<String>,
}

impl PayoutRequestBuilder {
    pub fn recipient(mut self, recipient: impl Into<String>) -> Self {
        self.recipient = Some(recipient.into());
        self
    }

    pub fn amount(mut self, amount: Amount) -> Self {
        self.amount = Some(amount);
        self
    }

    pub fn reference(mut self, reference: impl Into<String>) -> Self {
        self.reference = Some(reference.into());
        self
    }

    /// Sets the idempotency key. Without one, `build` derives a key from the
    /// recipient, reference and amount.
    pub fn idempotency_key(mut self, key: impl Into<String>) -> Self {
        self.idempotency_key = Some(key.into());
        self
    }

    /// Checks the request and builds it.
    pub fn build(self) -> Result<PayoutRequest, RequestError> {
        let recipient = self.recipient.ok_or(RequestError::MissingRecipient)?;
        let amount = self.amount.ok_or(RequestError::MissingAmount)?;
        let reference = self.reference.ok_or(RequestError::MissingReference)?;
        if amount.is_zero() {
            // The provider charges a fee per payout, empty ones included.
            return Err(RequestError::ZeroAmount);
        }
        let idempotency_key = self
            .idempotency_key
            .unwrap_or_else(|| format!("{recipient}:{reference}:{}", amount.canonical()));
        Ok(PayoutRequest {
            recipient,
            amount,
            reference,
            idempotency_key,
        })
    }
}

/// Why a [`PayoutRequest`] couldn't be built.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestError {
    MissingRecipient,
    MissingAmount,
    MissingReference,
    ZeroAmount,
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            RequestError::MissingRecipient => "payout request has no recipient",
            RequestError::MissingAmount => "payout request has no amount",
            RequestError::MissingReference => "payout request has no reference",
            RequestError::ZeroAmount => "payout amount is zero",
        })
    }
}

impl std::error::Error for RequestError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::money::Currency;

    #[test]
    fn builds_a_complete_request() {
        let req = PayoutRequest::builder()
            .recipient("acct_123")
            .amount(Amount::new(1250, Currency::Usd))
            .reference("INV-1001")
            .build()
            .unwrap();
        assert_eq!(req.recipient(), "acct_123");
        assert_eq!(req.reference(), "INV-1001");
    }

    #[test]
    fn an_explicit_idempotency_key_wins() {
        let req = PayoutRequest::builder()
            .recipient("acct_123")
            .amount(Amount::new(1250, Currency::Usd))
            .reference("INV-1001")
            .idempotency_key("idem-7f3a")
            .build()
            .unwrap();
        assert_eq!(req.idempotency_key(), "idem-7f3a");
    }

    #[test]
    fn requires_every_field() {
        let result = PayoutRequest::builder().recipient("acct_123").build();
        assert_eq!(result, Err(RequestError::MissingAmount));
    }

    #[test]
    fn rejects_zero_amounts() {
        let result = PayoutRequest::builder()
            .recipient("acct_123")
            .amount(Amount::new(0, Currency::Usd))
            .reference("INV-1001")
            .build();
        assert_eq!(result, Err(RequestError::ZeroAmount));
    }
}
