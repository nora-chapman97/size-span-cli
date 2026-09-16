//! Byte sizes: parsing strings like `1.5 MiB` or `2000` and formatting
//! them back out in a human-readable form.
//!
//! Both binary units (`KiB` = 1024, `MiB` = 1024^2, ...) and decimal units
//! (`kB` = 1000, `MB` = 1000^2, ...) are accepted on the way in, matched
//! case-insensitively. A bare number with no unit is treated as bytes.
//! There is no bit ("Kb", lowercase b meaning bit) support - `b`/`B` always
//! means bytes here.

use crate::parse_strict_number;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ByteSize(u64);

impl ByteSize {
    pub const fn from_bytes(bytes: u64) -> Self {
        ByteSize(bytes)
    }

    pub const fn as_bytes(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ByteSizeError {
    Empty,
    InvalidNumber(String),
    UnknownUnit(String),
    Overflow,
}

impl fmt::Display for ByteSizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ByteSizeError::Empty => write!(f, "input is empty"),
            ByteSizeError::InvalidNumber(s) => write!(f, "invalid number: {s:?}"),
            ByteSizeError::UnknownUnit(s) => write!(f, "unrecognized size or unit: {s:?}"),
            ByteSizeError::Overflow => write!(f, "value is too large to represent"),
        }
    }
}

impl std::error::Error for ByteSizeError {}

// Checked longest-suffix-first so "kib" is matched before "kb" before "b".
const UNITS: &[(&str, u64)] = &[
    ("pib", 1024u64.pow(5)),
    ("tib", 1024u64.pow(4)),
    ("gib", 1024u64.pow(3)),
    ("mib", 1024u64.pow(2)),
    ("kib", 1024),
    ("pb", 1000u64.pow(5)),
    ("tb", 1000u64.pow(4)),
    ("gb", 1000u64.pow(3)),
    ("mb", 1000u64.pow(2)),
    ("kb", 1000),
    ("b", 1),
];

impl FromStr for ByteSize {
    type Err = ByteSizeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(ByteSizeError::Empty);
        }
        let lower = trimmed.to_ascii_lowercase();

        let (num_part, multiplier) = match UNITS.iter().find(|(unit, _)| lower.ends_with(unit)) {
            Some(&(unit, multiplier)) => (trimmed[..trimmed.len() - unit.len()].trim(), multiplier),
            None => (trimmed, 1),
        };

        let value = parse_strict_number(num_part)
            .ok_or_else(|| ByteSizeError::InvalidNumber(num_part.to_string()))?;

        let scaled = value * multiplier as f64;
        if !scaled.is_finite() || scaled < 0.0 || scaled > u64::MAX as f64 {
            return Err(ByteSizeError::Overflow);
        }
        Ok(ByteSize(scaled.round() as u64))
    }
}

impl fmt::Display for ByteSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const STEPS: [(&str, u64); 5] = [
            ("PiB", 1024u64.pow(5)),
            ("TiB", 1024u64.pow(4)),
            ("GiB", 1024u64.pow(3)),
            ("MiB", 1024u64.pow(2)),
            ("KiB", 1024),
        ];
        for (name, size) in STEPS {
            if self.0 >= size {
                let value = self.0 as f64 / size as f64;
                let text = format!("{value:.2}");
                let text = text.trim_end_matches('0').trim_end_matches('.');
                return write!(f, "{text} {name}");
            }
        }
        write!(f, "{} B", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_sizes() {
        let cases: &[(&str, u64)] = &[
            ("0", 0),
            ("0B", 0),
            ("5", 5),
            ("5B", 5),
            ("1024", 1024),
            ("1KiB", 1024),
            ("1 KiB", 1024),
            ("1kib", 1024),
            ("1KIB", 1024),
            ("1kB", 1000),
            ("1kb", 1000),
            ("1.5MiB", 1_572_864),
            ("1.5MB", 1_500_000),
            ("2GiB", 2 * 1024 * 1024 * 1024),
            ("1TiB", 1024u64.pow(4)),
            (".5KiB", 512),
            ("1024.", 1024),
        ];
        for (input, expected) in cases {
            match ByteSize::from_str(input) {
                Ok(size) => assert_eq!(size.as_bytes(), *expected, "input: {input:?}"),
                Err(e) => panic!("expected {input:?} to parse as {expected}, got error: {e}"),
            }
        }
    }

    #[test]
    fn rejects_invalid_sizes() {
        let cases: &[&str] = &[
            "",
            "   ",
            "-1B",
            "-1",
            "1XB",
            "MB",
            "B",
            "1e3MB",
            "1.5.3MB",
            "1,000B",
            "inf",
            "infB",
            "nan",
            "1..5MB",
            "+5MB",
        ];
        for input in cases {
            assert!(
                ByteSize::from_str(input).is_err(),
                "expected {input:?} to fail to parse"
            );
        }
    }

    #[test]
    fn overflow_is_rejected() {
        assert!(ByteSize::from_str("999999999999999999999999TiB").is_err());
    }

    #[test]
    fn formats_sizes() {
        let cases: &[(u64, &str)] = &[
            (0, "0 B"),
            (5, "5 B"),
            (1023, "1023 B"),
            (1024, "1 KiB"),
            (1536, "1.5 KiB"),
            (1_048_576, "1 MiB"),
            (1_572_864, "1.5 MiB"),
        ];
        for (bytes, expected) in cases {
            assert_eq!(
                ByteSize::from_bytes(*bytes).to_string(),
                *expected,
                "bytes: {bytes}"
            );
        }
    }
}
