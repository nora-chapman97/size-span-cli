//! Durations: parsing strings like `1h30m` or `500ms` and formatting a
//! `std::time::Duration` back out the same way.
//!
//! Unlike byte sizes, a unit is always required here - there is no sane
//! default unit for a bare number, so `"100"` is rejected rather than
//! guessed at. Units must appear largest to smallest (`d`, `h`, `m`, `s`,
//! `ms`) with no repeats and no whitespace between components, e.g.
//! `1d2h3m4s5ms` is fine but `1h1h` and `1m1h` are both errors.

use crate::parse_strict_number;
use std::fmt;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DurationError {
    Empty,
    MissingNumber(String),
    MissingUnit(String),
    UnknownUnit(String),
    InvalidNumber(String),
    DuplicateUnit(String),
    OutOfOrder(String),
    Overflow,
}

impl fmt::Display for DurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DurationError::Empty => write!(f, "input is empty"),
            DurationError::MissingNumber(s) => write!(f, "expected a number, found: {s:?}"),
            DurationError::MissingUnit(s) => {
                write!(f, "missing unit after {s:?} (expected d, h, m, s, or ms)")
            }
            DurationError::UnknownUnit(s) => write!(f, "unrecognized duration unit: {s:?}"),
            DurationError::InvalidNumber(s) => write!(f, "invalid number: {s:?}"),
            DurationError::DuplicateUnit(s) => write!(f, "unit {s:?} used more than once"),
            DurationError::OutOfOrder(s) => write!(
                f,
                "unit {s:?} is out of order (units must go from largest to smallest: d, h, m, s, ms)"
            ),
            DurationError::Overflow => write!(f, "duration is too large to represent"),
        }
    }
}

impl std::error::Error for DurationError {}

// (rank, milliseconds per unit); rank must strictly increase between
// components of a compound string.
fn unit_info(unit: &str) -> Option<(u8, u64)> {
    match unit {
        "d" => Some((0, 86_400_000)),
        "h" => Some((1, 3_600_000)),
        "m" => Some((2, 60_000)),
        "s" => Some((3, 1_000)),
        "ms" => Some((4, 1)),
        _ => None,
    }
}

pub fn parse_duration(s: &str) -> Result<Duration, DurationError> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err(DurationError::Empty);
    }

    let bytes = trimmed.as_bytes();
    let mut i = 0;
    let mut total_ms: u128 = 0;
    let mut last_rank: Option<u8> = None;

    while i < bytes.len() {
        let num_start = i;
        while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
            i += 1;
        }
        if i == num_start {
            return Err(DurationError::MissingNumber(trimmed[i..].to_string()));
        }
        let num_str = &trimmed[num_start..i];
        let value = parse_strict_number(num_str)
            .ok_or_else(|| DurationError::InvalidNumber(num_str.to_string()))?;

        let unit_start = i;
        while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
            i += 1;
        }
        if i == unit_start {
            return Err(DurationError::MissingUnit(num_str.to_string()));
        }
        let unit = &trimmed[unit_start..i];
        let unit_lower = unit.to_ascii_lowercase();
        let (rank, ms_per_unit) =
            unit_info(&unit_lower).ok_or_else(|| DurationError::UnknownUnit(unit.to_string()))?;

        if let Some(last) = last_rank {
            if rank == last {
                return Err(DurationError::DuplicateUnit(unit.to_string()));
            }
            if rank < last {
                return Err(DurationError::OutOfOrder(unit.to_string()));
            }
        }
        last_rank = Some(rank);

        let ms = value * ms_per_unit as f64;
        total_ms += ms.round() as u128;
    }

    if total_ms > u64::MAX as u128 {
        return Err(DurationError::Overflow);
    }
    Ok(Duration::from_millis(total_ms as u64))
}

pub fn format_duration(d: Duration) -> String {
    let mut remaining = d.as_millis();
    if remaining == 0 {
        return "0s".to_string();
    }

    let mut out = String::new();
    for (name, ms_per_unit) in [
        ("d", 86_400_000u128),
        ("h", 3_600_000),
        ("m", 60_000),
        ("s", 1_000),
    ] {
        let count = remaining / ms_per_unit;
        remaining %= ms_per_unit;
        if count > 0 {
            out.push_str(&count.to_string());
            out.push_str(name);
        }
    }
    if remaining > 0 {
        out.push_str(&remaining.to_string());
        out.push_str("ms");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_durations() {
        let cases: &[(&str, u64)] = &[
            ("0s", 0),
            ("1d", 86_400_000),
            ("1h", 3_600_000),
            ("1h30m", 5_400_000),
            ("90m", 5_400_000),
            ("1.5h", 5_400_000),
            ("500ms", 500),
            ("1s500ms", 1_500),
            ("1d2h3m4s5ms", 93_784_005),
            (".5s", 500),
        ];
        for (input, expected_ms) in cases {
            match parse_duration(input) {
                Ok(d) => assert_eq!(d.as_millis() as u64, *expected_ms, "input: {input:?}"),
                Err(e) => panic!("expected {input:?} to parse as {expected_ms}ms, got error: {e}"),
            }
        }
    }

    #[test]
    fn rejects_invalid_durations() {
        let cases: &[&str] = &[
            "",
            "   ",
            "h",
            "100",
            "1y",
            "-1s",
            "1h1h",
            "1m1h",
            "1h 30m",
            "1..5s",
            "1e3s",
        ];
        for input in cases {
            assert!(
                parse_duration(input).is_err(),
                "expected {input:?} to fail to parse"
            );
        }
    }

    #[test]
    fn overflow_is_rejected() {
        assert!(parse_duration("99999999999999999999999999d").is_err());
    }

    #[test]
    fn formats_durations() {
        let cases: &[(u64, &str)] = &[
            (0, "0s"),
            (500, "500ms"),
            (1_500, "1s500ms"),
            (5_400_000, "1h30m"),
            (93_784_005, "1d2h3m4s5ms"),
        ];
        for (ms, expected) in cases {
            assert_eq!(
                format_duration(Duration::from_millis(*ms)),
                *expected,
                "ms: {ms}"
            );
        }
    }
}
