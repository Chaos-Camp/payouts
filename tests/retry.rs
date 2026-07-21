//! The retry loop, end to end, against a scripted provider.

use payouts::{
    send_with_config, send_with_retry, Amount, Currency, FakeProvider, NoSleep, PayoutRequest,
    ProviderError, RetryConfig, RetryError,
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
fn gives_up_after_five_attempts() {
    let provider = FakeProvider::always_failing(503);
    let config = RetryConfig::default();
    let result = send_with_config(&provider, &request(), &config, &mut NoSleep);
    assert_eq!(
        result,
        Err(RetryError::Exhausted {
            attempts: 5,
            last: ProviderError::Transient(503),
        })
    );
    assert_eq!(provider.calls(), 5);
}

#[test]
fn returns_the_receipt() {
    let provider = FakeProvider::succeeding();
    let receipt = send_with_retry(&provider, &request(), 3, &mut NoSleep).unwrap();
    assert_eq!(receipt.id, "po_0001");
    assert_eq!(receipt.reference, "INV-1001");
    assert_eq!(receipt.amount, Amount::new(1250, Currency::Usd));
}

#[test]
fn passes_a_fatal_error_through() {
    let provider = FakeProvider::fatal("recipient account closed");
    let result = send_with_retry(&provider, &request(), 10, &mut NoSleep);
    let expected = ProviderError::Fatal("recipient account closed".to_string());
    assert_eq!(result, Err(RetryError::Fatal(expected)));
    assert_eq!(provider.calls(), 1, "fatal errors must not be retried");
}
