# ext-time

A Rust library that extends the [time](https://crates.io/crates/time) crate with additional useful methods for time manipulation and formatting.

## Features

- Extension methods for `Time` type:
  - Format time as HH:MM (`to_shorten`)
  - Parse time from HH:MM string (`from_str`)
  - Duration calculation with cross-day handling (`sub_ext`)
  - Time range checks (`is_between`)
  - Minute-based operations (`reset_minute`, `is_same_minute`, `add_minutes`)

- Extension methods for `OffsetDateTime`:
  - Timestamp conversions (milliseconds/seconds)
  - Timezone-aware formatting (display/Chinese format)
  - Parse from various formats (milliseconds, date-time string, simple format)
  - Minute-based operations

- Serde support for `OffsetDateTime`:
  - Serialize to Unix timestamp
  - Deserialize from Unix timestamp

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ext-time = "0.1.0"
```

## Usage

```rust
use ext_time::prelude::*;
use time::macros::time;

// Time operations
let t = time!(23:30);
assert_eq!(t.to_shorten(), "23:30");
assert!(t.is_between(time!(23:00), time!(0:00)));
let next = t.add_minutes(40); // Handles midnight wraparound

// OffsetDateTime with timezone
let dt = OffsetDateTime::now_with_offset(8); // UTC+8
println!("{}", dt.to_display_string(8)); // "2024-03-20 15:30:45.123+08:00"
println!("{}", dt.to_chinese_string()); // "2024年03月20日 15时30分45秒 +8"

// Parse from various formats
let dt = OffsetDateTime::from_milliseconds(1694696340500, 8)?;
let dt = OffsetDateTime::from_date_time("20240320", "15:30:45", 123, 8)?;
let dt = OffsetDateTime::from_simple("20240320_1530", 8)?;

// Serde support
#[derive(Serialize, Deserialize)]
struct Event {
    #[serde(serialize_with = "serde_t2ts", deserialize_with = "serde_parse_ts")]
    timestamp: OffsetDateTime,
}
```

## Documentation

For detailed documentation and examples, please check the [API documentation](https://docs.rs/ext-time).

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

