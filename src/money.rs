//! Money, stored as integer minor units.
//!
//! Floating point never touches an amount. `12.50 USD` is held as 1250
//! cents; `500 JPY` is held as 500 yen, because the yen has no minor unit.
//! Arithmetic is checked, so an overflow is an error you can see rather
//! than a payout you can't explain.

use std::fmt;
use std::str::FromStr;

/// The currencies Tidepool pays out in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Currency {
    Usd,
    Eur,
    Gbp,
    Jpy,
}

impl Currency {
    /// The ISO 4217 code, e.g. `"USD"`.
    pub fn code(self) -> &'static str {
        match self {
            Currency::Usd => "USD",
            Currency::Eur => "EUR",
            Currency::Gbp => "GBP",
            Currency::Jpy => "JPY",
        }
    }

    /// How many decimal places the minor unit has: 2 for cents, 0 for yen.
    pub fn exponent(self) -> u32 {
        match self {
            Currency::Jpy => 0,
            _ => 2,
        }
    }
}

impl FromStr for Currency {
    type Err = MoneyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "USD" => Ok(Currency::Usd),
            "EUR" => Ok(Currency::Eur),
            "GBP" => Ok(Currency::Gbp),
            "JPY" => Ok(Currency::Jpy),
            other => Err(MoneyError::UnknownCurrency(other.to_string())),
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

    /// Adds two amounts of the same currency.
    ///
    /// Mixing currencies is an error, not an implicit conversion, and so is
    /// overflowing `u64`.
    pub fn checked_add(self, other: Amount) -> Result<Amount, MoneyError> {
        if self.currency != other.currency {
            return Err(MoneyError::CurrencyMismatch {
                left: self.currency,
                right: other.currency,
            });
        }
        let minor = self
            .minor
            .checked_add(other.minor)
            .ok_or(MoneyError::Overflow)?;
        Ok(Amount::new(minor, self.currency))
    }
}

impl FromStr for Amount {
    type Err = MoneyError;

    /// Parses `"<number> <CODE>"`, e.g. `"12.50 USD"` or `"500 JPY"`.
    ///
    /// The number may have fewer decimal places than the currency allows
    /// (`"12.5 USD"` is $12.50) but never more: `"12.505 USD"` is rejected
    /// rather than silently rounded.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let invalid = || MoneyError::Invalid(s.to_string());

        let (number, code) = s.trim().split_once(' ').ok_or_else(invalid)?;
        let currency: Currency = code.parse()?;
        let exponent = currency.exponent();

        let (whole, frac) = number.split_once('.').unwrap_or((number, ""));
        let digits_only = |part: &str| part.bytes().all(|b| b.is_ascii_digit());
        if whole.is_empty() || !digits_only(whole) || !digits_only(frac) {
            return Err(invalid());
        }
        if frac.len() > exponent as usize || (number.contains('.') && frac.is_empty()) {
            return Err(invalid());
        }

        // "12.5" in a two-decimal currency is 12 * 100 + 5 * 10.
        let scale = 10u64.pow(exponent);
        let frac_scale = 10u64.pow(exponent - frac.len() as u32);
        // Both parts are known to be digits, so parsing can only fail on
        // overflow, or on an empty fraction, which means zero.
        let whole: u64 = whole.parse().map_err(|_| MoneyError::Overflow)?;
        let frac: u64 = frac.parse().unwrap_or(0);

        let minor = whole
            .checked_mul(scale)
            .and_then(|w| w.checked_add(frac * frac_scale))
            .ok_or(MoneyError::Overflow)?;
        Ok(Amount::new(minor, currency))
    }
}

impl fmt::Display for Amount {
    /// Formats as `"12.50 USD"`, the same shape `parse` accepts.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let exponent = self.currency.exponent();
        if exponent == 0 {
            return write!(f, "{} {}", self.minor, self.currency);
        }
        let scale = 10u64.pow(exponent);
        write!(
            f,
            "{}.{:0width$} {}",
            self.minor / scale,
            self.minor % scale,
            self.currency,
            width = exponent as usize
        )
    }
}

/// Why an amount couldn't be parsed or computed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoneyError {
    /// The text wasn't `"<number> <CODE>"`.
    Invalid(String),
    UnknownCurrency(String),
    CurrencyMismatch {
        left: Currency,
        right: Currency,
    },
    Overflow,
}

impl fmt::Display for MoneyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MoneyError::Invalid(s) => write!(f, "invalid amount {s:?}"),
            MoneyError::UnknownCurrency(code) => write!(f, "unknown currency {code:?}"),
            MoneyError::CurrencyMismatch { left, right } => {
                write!(f, "cannot combine {left} with {right}")
            }
            MoneyError::Overflow => f.write_str("amount overflows u64 minor units"),
        }
    }
}

impl std::error::Error for MoneyError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_formats_round_trip() {
        for text in ["12.50 USD", "0.05 EUR", "1000.00 GBP", "500 JPY"] {
            let amount: Amount = text.parse().unwrap();
            assert_eq!(amount.to_string(), text);
        }
    }

    #[test]
    fn pads_short_fractions() {
        let amount: Amount = "12.5 USD".parse().unwrap();
        assert_eq!(amount, Amount::new(1250, Currency::Usd));
    }

    #[test]
    fn rejects_malformed_amounts() {
        let malformed = ["", "12.50", "12.505 USD", "-1.00 USD", "12. USD", "1.5 JPY"];
        for text in malformed {
            assert!(text.parse::<Amount>().is_err(), "{text:?} should not parse");
        }
        assert_eq!(
            "1.00 XYZ".parse::<Amount>(),
            Err(MoneyError::UnknownCurrency("XYZ".to_string()))
        );
    }

    #[test]
    fn checked_add_refuses_mixed_currencies() {
        let usd = Amount::new(100, Currency::Usd);
        let eur = Amount::new(100, Currency::Eur);
        let err = usd.checked_add(eur).unwrap_err();
        assert!(matches!(err, MoneyError::CurrencyMismatch { .. }));
    }

    #[test]
    fn checked_add_reports_overflow() {
        let big = Amount::new(u64::MAX, Currency::Usd);
        let one = Amount::new(1, Currency::Usd);
        assert_eq!(big.checked_add(one), Err(MoneyError::Overflow));
    }
}
