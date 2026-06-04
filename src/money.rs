//! Money, stored as integer minor units.
//!
//! Floating point never touches an amount. `12.50 USD` is held as 1250
//! cents.

use std::fmt;

/// The currencies Tidepool pays out in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Currency {
    Usd,
    Eur,
    Gbp,
}

impl Currency {
    /// The ISO 4217 code, e.g. `"USD"`.
    pub fn code(self) -> &'static str {
        match self {
            Currency::Usd => "USD",
            Currency::Eur => "EUR",
            Currency::Gbp => "GBP",
        }
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

/// An amount of money in a single currency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Amount {
    minor: u64,
    currency: Currency,
}

impl Amount {
    /// `minor` units of `currency`: `Amount::new(1250, Currency::Usd)` is $12.50.
    pub const fn new(minor: u64, currency: Currency) -> Self {
        Amount { minor, currency }
    }

    /// The amount in minor units (cents, pence, yen).
    pub fn minor(self) -> u64 {
        self.minor
    }

    pub fn currency(self) -> Currency {
        self.currency
    }
}

impl fmt::Display for Amount {
    /// Formats as `"12.50 USD"`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{:02} {}",
            self.minor / 100,
            self.minor % 100,
            self.currency
        )
    }
}
