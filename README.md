# sizespan

A small Rust library (with a thin CLI on top) for parsing and formatting
byte sizes and durations. No dependencies beyond the standard library.

The problem it solves: every project ends up needing to turn `"1.5 MiB"`
into a byte count, or `"1h30m"` into a duration, and the naive version of
that parser is always missing a handful of cases - `kb` vs `kib`, a unit
with no number, units in the wrong order, a value too large to fit in a
`u64`. This crate is my attempt to get that logic right once, with a test
suite that names the awkward cases instead of just the happy path.

## Byte sizes

`sizespan::bytes::ByteSize` implements `FromStr` and `Display`.

```rust
use sizespan::bytes::ByteSize;
use std::str::FromStr;

let size = ByteSize::from_str("1.5 MiB").unwrap();
assert_eq!(size.as_bytes(), 1_572_864);

assert_eq!(ByteSize::from_bytes(1536).to_string(), "1.5 KiB");
```

Both binary units (`KiB` = 1024, `MiB` = 1024^2, ...) and decimal units
(`kB` = 1000, `MB` = 1000^2, ...) are accepted, matched case-insensitively.
A bare number with no unit (`"2048"`) is treated as bytes. There is no bit
support - `b`/`B` always means bytes, never bits, regardless of case.

## Durations

`sizespan::duration` exposes free functions instead of a `FromStr` impl,
since `std::time::Duration` is a foreign type and can't have one added to
it from outside its own crate.

```rust
use sizespan::duration::{parse_duration, format_duration};

let d = parse_duration("1h30m").unwrap();
assert_eq!(d.as_secs(), 5400);
assert_eq!(format_duration(d), "1h30m");
```

Unlike byte sizes, a unit is always required - there's no sensible default
unit for a bare number, so `"100"` is rejected rather than guessed at.
Units (`d`, `h`, `m`, `s`, `ms`) must appear largest to smallest with no
repeats and no internal whitespace: `1d2h3m4s5ms` parses, but `1h1h` and
`1m1h` are both errors.

## CLI

```
sizespan bytes parse 1.5MiB      -> 1572864 bytes
sizespan bytes fmt 1572864       -> 1.5 MiB
sizespan duration parse 1h30m    -> 5400000 ms
sizespan duration fmt 5400       -> 1h30m
```

## Status

Early. The parsing and formatting core is here with a table-driven test
suite covering the edge cases above; the CLI is intentionally minimal.
