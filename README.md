# payouts

Send payouts to a payment provider, and retry the ones that fail for reasons
worth retrying.

`payouts` is the small library Tidepool uses between "we owe this person
money" and "the provider says it's sent". It knows three things:

- **Money** is integer minor units in one currency. `12.50 USD` is 1250
  cents. There are no floats anywhere.
- **A provider** either accepts a payout and returns a `Receipt`, or fails
  with an error that is `Transient` (a 429 or 5xx: try again) or `Fatal`
  (a closed account: don't).
- **Retrying** waits between attempts with exponential backoff: 1s, 2s, 4s,
  and so on, up to `RetryConfig::max_attempts` attempts in total.

No dependencies beyond `std`.

## Example

```rust
use payouts::{send_with_config, Amount, PayoutRequest, RetryConfig, ThreadSleeper};

let req = PayoutRequest::builder()
    .recipient("acct_8812")
    .amount("12.50 USD".parse::<Amount>()?)
    .reference("INV-1001")
    .build()?;

let receipt = send_with_config(&provider, &req, &RetryConfig::default(), &mut ThreadSleeper)?;
println!("paid {} as {}", receipt.amount, receipt.id);
```

`provider` is anything that implements `payouts::Provider`. In tests, use
`FakeProvider`, which fails a scripted number of times and then succeeds,
together with `NoSleep` so the retry loop doesn't really wait.

## Development

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

CI runs the same three.

---

Part of the Tidepool demo for [diff social](https://diffsocial.dev). Tidepool
is a fictional payments company; the provider, the people and the bugs are
made up.

License: MIT
