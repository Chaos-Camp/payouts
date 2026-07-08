//! The retry loop, end to end, against a scripted provider.

use payouts::{
    send_with_retry, Amount, Currency, FakeProvider, NoSleep, PayoutRequest, ProviderError,
    RetryError,
};

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
    let result = send_with_retry(&provider, &request(), 5, &mut NoSleep);
    assert!(result.is_ok());
    assert_eq!(provider.calls(), 3);
}

#[test]
fn passes_a_fatal_error_through() {
    let provider = FakeProvider::fatal("recipient account closed");
    let result = send_with_retry(&provider, &request(), 10, &mut NoSleep);
    let expected = ProviderError::Fatal("recipient account closed".to_string());
    assert_eq!(result, Err(RetryError::Fatal(expected)));
    assert_eq!(provider.calls(), 1, "fatal errors must not be retried");
}
