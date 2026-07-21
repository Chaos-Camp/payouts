//! Retrying payouts against a provider that sometimes says "not now".
//!
//! [`send_with_retry`] calls [`Provider::send`] until it succeeds, the
//! provider returns a fatal error, or the attempt budget runs out. Between
//! attempts it waits [`backoff`]`(attempt)`, which doubles each time so a
//! struggling provider gets room to recover.
//!
//! Waiting goes through the [`Sleeper`] trait so that tests can run the
//! whole retry loop without actually sleeping.

use std::fmt;
use std::thread;
use std::time::Duration;

use crate::config::RetryConfig;
use crate::provider::{Provider, ProviderError, Receipt};
use crate::request::PayoutRequest;

/// The wait after the first failed attempt.
const BASE_DELAY: Duration = Duration::from_secs(1);

/// Something that can wait. Tests swap in [`NoSleep`].
pub trait Sleeper {
    fn sleep(&mut self, duration: Duration);
}

/// Blocks the current thread.
#[derive(Debug, Default, Clone, Copy)]
pub struct ThreadSleeper;

impl Sleeper for ThreadSleeper {
    fn sleep(&mut self, duration: Duration) {
        thread::sleep(duration);
    }
}

/// Doesn't wait at all. For tests.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoSleep;

impl Sleeper for NoSleep {
    fn sleep(&mut self, _duration: Duration) {}
}

/// How long to wait after attempt number `attempt` fails.
///
/// The wait starts at one second and doubels with each failure: 1s, 2s,
/// 4s, 8s, ...
pub fn backoff(attempt: u32) -> Duration {
    BASE_DELAY * 2u32.pow(attempt)
}

/// Why [`send_with_retry`] gave up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetryError {
    /// Every attempt failed with a transient error.
    Exhausted {
        attempts: u32,
        /// The error from the final attempt.
        last: ProviderError,
    },
    /// The provider returned an error that retrying can't fix.
    Fatal(ProviderError),
}

impl fmt::Display for RetryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RetryError::Exhausted { attempts, last } => {
                write!(f, "gave up after {attempts} attempts: {last}")
            }
            RetryError::Fatal(err) => write!(f, "not retrying: {err}"),
        }
    }
}

impl std::error::Error for RetryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RetryError::Exhausted { last, .. } => Some(last),
            RetryError::Fatal(err) => Some(err),
        }
    }
}

/// Sends `req`, making up to `max_attempts` attempts in total.
///
/// After each transient failure it sleeps for [`backoff`] of the attempt
/// that just failed. Fatal errors are returned at once: there is no point
/// asking twice for a payout to a closed account.
///
/// `max_attempts` counts the first try, so `1` means "no retries". `0` is
/// treated as `1`; a payout is always attempted at least once.
pub fn send_with_retry<P, S>(
    provider: &P,
    req: &PayoutRequest,
    max_attempts: u32,
    sleeper: &mut S,
) -> Result<Receipt, RetryError>
where
    P: Provider + ?Sized,
    S: Sleeper + ?Sized,
{
    let max_attempts = max_attempts.max(1);
    let mut last_error = None;
    for attempt in 0..max_attempts {
        match provider.send(req) {
            Ok(receipt) => return Ok(receipt),
            Err(err) if !err.is_retryable() => return Err(RetryError::Fatal(err)),
            Err(err) => last_error = Some(err),
        }
        sleeper.sleep(backoff(attempt));
    }
    Err(RetryError::Exhausted {
        attempts: max_attempts,
        last: last_error.expect("max_attempts is at least 1"),
    })
}

/// [`send_with_retry`], with the attempt budget taken from `config`.
pub fn send_with_config<P, S>(
    provider: &P,
    req: &PayoutRequest,
    config: &RetryConfig,
    sleeper: &mut S,
) -> Result<Receipt, RetryError>
where
    P: Provider + ?Sized,
    S: Sleeper + ?Sized,
{
    send_with_retry(provider, req, config.max_attempts, sleeper)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::money::{Amount, Currency};
    use crate::provider::FakeProvider;

    /// Counts how many times it was asked to sleep.
    #[derive(Default)]
    struct CountingSleeper {
        sleeps: u32,
    }

    impl Sleeper for CountingSleeper {
        fn sleep(&mut self, _duration: Duration) {
            self.sleeps += 1;
        }
    }

    fn request() -> PayoutRequest {
        PayoutRequest::builder()
            .recipient("acct_123")
            .amount(Amount::new(1250, Currency::Usd))
            .reference("INV-1001")
            .build()
            .unwrap()
    }

    #[test]
    fn backoff_doubles_from_one_second() {
        let waits: Vec<u64> = (0..5).map(|a| backoff(a).as_secs()).collect();
        assert_eq!(waits, [1, 2, 4, 8, 16]);
    }

    #[test]
    fn sleeps_once_per_failed_attempt() {
        let provider = FakeProvider::always_failing(503);
        let mut sleeper = CountingSleeper::default();
        let _ = send_with_retry(&provider, &request(), 4, &mut sleeper);
        assert_eq!(provider.calls(), 4);
        assert_eq!(sleeper.sleeps, 4);
    }

    #[test]
    fn zero_attempts_still_tries_once() {
        let provider = FakeProvider::succeeding();
        assert!(send_with_retry(&provider, &request(), 0, &mut NoSleep).is_ok());
        assert_eq!(provider.calls(), 1);
    }
}
