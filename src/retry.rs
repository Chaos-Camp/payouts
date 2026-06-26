//! Retrying payouts against a provider that sometimes says "not now".
//!
//! [`send_with_retry`] calls [`Provider::send`] until it succeeds, the
//! provider returns a fatal error, or the attempt budget runs out. Between
//! attempts it waits [`backoff`]`(attempt)`, which doubles each time so a
//! struggling provider gets room to recover.
//!
//! Waiting goes through the [`Sleeper`] trait so that tests can run the
//! whole retry loop without actually sleeping.

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

/// Sends `req`, making up to `max_attempts` attempts in total.
///
/// After each transient failure it sleeps for [`backoff`] of the attempt
/// that just failed. Fatal errors are returned at once: there is no point
/// asking twice for a payout to a closed account.
///
/// `max_attempts` counts the first try, so `1` means "no retries".
pub fn send_with_retry<P, S>(
    provider: &P,
    req: &PayoutRequest,
    max_attempts: u32,
    sleeper: &mut S,
) -> Result<Receipt, ProviderError>
where
    P: Provider + ?Sized,
    S: Sleeper + ?Sized,
{
    let mut last_error = None;
    for attempt in 0..max_attempts {
        match provider.send(req) {
            Ok(receipt) => return Ok(receipt),
            Err(err) if !err.is_retryable() => return Err(err),
            Err(err) => last_error = Some(err),
        }
        sleeper.sleep(backoff(attempt));
    }
    Err(last_error.expect("at least one attempt was made"))
}

/// [`send_with_retry`], with the attempt budget taken from `config`.
pub fn send_with_config<P, S>(
    provider: &P,
    req: &PayoutRequest,
    config: &RetryConfig,
    sleeper: &mut S,
) -> Result<Receipt, ProviderError>
where
    P: Provider + ?Sized,
    S: Sleeper + ?Sized,
{
    send_with_retry(provider, req, config.max_attempts, sleeper)
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
