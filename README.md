# Ext Time

A Rust library providing extension traits for time manipulation, built on top of the `time` crate.

## Features

- Extension traits for `Time` and `OffsetDateTime`
- Time formatting and parsing utilities
- Time alignment and rounding functions
- Time arithmetic operations
- Timezone handling
- Date boundary calculations (start/end of day, week, month)

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
ext-time = "0.1.0"
```

## Examples

```rust
use ext_time::{ExtTime, ExtOffsetDateTime};
use time::{Time, OffsetDateTime};

// Time operations
let time = Time::from_hms(14, 30, 45).unwrap();
let next_hour = time.next_hour();
let aligned = time.align_to(300).unwrap(); // Align to 5 minutes

// DateTime operations
let dt = OffsetDateTime::now_utc();
let start_of_day = dt.start_of_day();
let end_of_month = dt.end_of_month();
```

## License

MIT
