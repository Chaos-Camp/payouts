//! Tunables for the retry loop.

/// How hard [`crate::send_with_retry`] tries before giving up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryConfig {
    /// Total attempts, including the first. Always at least 1.
    pub max_attempts: u32,
}

impl RetryConfig {
    /// Enough to ride out a short blip.
    pub const DEFAULT_MAX_ATTEMPTS: u32 = 5;

    pub fn new(max_attempts: u32) -> Self {
        RetryConfig {
            max_attempts: max_attempts.max(1),
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        RetryConfig::new(Self::DEFAULT_MAX_ATTEMPTS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_five_attempts() {
        assert_eq!(RetryConfig::default().max_attempts, 5);
    }

    #[test]
    fn never_allows_zero_attempts() {
        assert_eq!(RetryConfig::new(0).max_attempts, 1);
    }
}
