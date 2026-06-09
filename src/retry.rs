//! Retrying payouts against a provider that sometimes says "not now".
//!
//! [`send_with_retry`] calls [`Provider::send`] until it succeeds or
//! the attempt budget runs out. Between
//! attempts it waits [`backoff`]`(attempt)`, which doubles each time so a
//! struggling provider gets room to recover.

use std::thread;
use std::time::Duration;

use crate::provider::{Provider, ProviderError, Receipt};
use crate::request::PayoutRequest;

/// The wait after the first failed attempt.
const BASE_DELAY: Duration = Duration::from_secs(1);

/// How long to wait after attempt number `attempt` fails.
///
/// The wait starts at one second and doubels with each failure: 1s, 2s,
/// 4s, 8s, ...
pub fn backoff(attempt: u32) -> Duration {
    BASE_DELAY * 2u32.pow(attempt)
}

/// Sends `req`, making up to `max_attempts` attempts in total.
///
/// After each failure it sleeps for [`backoff`] of the attempt that just
/// failed.
///
/// `max_attempts` counts the first try, so `1` means "no retries".
pub fn send_with_retry<P>(
    provider: &P,
    req: &PayoutRequest,
    max_attempts: u32,
) -> Result<Receipt, ProviderError>
where
    P: Provider + ?Sized,
{
    let mut last_error = None;
    for attempt in 0..max_attempts {
        match provider.send(req) {
            Ok(receipt) => return Ok(receipt),
            Err(err) => last_error = Some(err),
        }
        thread::sleep(backoff(attempt));
    }
    Err(last_error.expect("at least one attempt was made"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_doubles_from_one_second() {
        let waits: Vec<u64> = (0..5).map(|a| backoff(a).as_secs()).collect();
        assert_eq!(waits, [1, 2, 4, 8, 16]);
    }
}
