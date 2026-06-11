//! The retry loop, end to end, against a scripted provider.

use payouts::{send_with_retry, Amount, Currency, FakeProvider, PayoutRequest};

fn request() -> PayoutRequest {
    PayoutRequest::builder()
        .recipient("acct_8812")
        .amount(Amount::new(1250, Currency::Usd))
        .reference("INV-1001")
        .build()
        .expect("a complete request")
}

#[test]
fn retries_a_failed_payout() {
    let provider = FakeProvider::failing(2, 503);
    // Really sleeps: 1s, then 2s.
    let result = send_with_retry(&provider, &request(), 5);
    assert!(result.is_ok());
    assert_eq!(provider.calls(), 3);
}
