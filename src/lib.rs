//! Parsing and formatting for byte sizes and durations.
//!
//! Two small, independent modules: `bytes` for quantities like `1.5 MiB`,
//! and `duration` for spans like `1h30m`. See each module's tests for the
//! exact set of inputs that are accepted and rejected.

pub mod bytes;
pub mod duration;

/// Parses a numeral that is only digits and at most one `.`.
///
/// This is stricter than `f64::from_str`, which also accepts things like
/// `inf`, `nan`, `5e3`, and a leading `+`/`-`. None of those make sense as a
/// byte count or a duration component, so both modules reject them by
/// routing their numeral through here first.
pub(crate) fn parse_strict_number(s: &str) -> Option<f64> {
    if s.is_empty() {
        return None;
    }
    let mut dot_seen = false;
    let mut digit_seen = false;
    for c in s.chars() {
        if c.is_ascii_digit() {
            digit_seen = true;
        } else if c == '.' && !dot_seen {
            dot_seen = true;
        } else {
            return None;
        }
    }
    if !digit_seen {
        return None;
    }
    s.parse::<f64>().ok()
}
