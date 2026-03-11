//! Timestamp formatting for output line prefixes.

use chrono::{DateTime, Local, NaiveDate};
use std::time::Duration;

/// The default timestamp format string (chrono syntax): `HH:MM:SS.mmm`.
pub const DEFAULT_FORMAT: &str = "%H:%M:%S%.3f";

/// Format a timestamp prefix string.
///
/// Uses `fmt` as a chrono format string. When `from_zero` is true, the time
/// base is `00:00:00.000` plus the elapsed duration instead of wall-clock time.
pub fn format_timestamp(
    fmt: &str,
    elapsed: Duration,
    wall_clock: DateTime<Local>,
    from_zero: bool,
) -> String {
    let formatted = if from_zero {
        let base = NaiveDate::from_ymd_opt(2000, 1, 1)
            .expect("valid date")
            .and_hms_opt(0, 0, 0)
            .expect("valid time");
        let t = base + chrono::Duration::from_std(elapsed).unwrap_or(chrono::Duration::MAX);
        t.format(fmt).to_string()
    } else {
        wall_clock.format(fmt).to_string()
    };
    format!("[{formatted}]")
}

/// Format a duration as `X.XXXs` (e.g., `1.234s`, `0.001s`, `123.456s`).
pub fn format_duration(d: Duration) -> String {
    format!("{:.3}s", d.as_secs_f64())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn test_time() -> DateTime<Local> {
        Local
            .with_ymd_and_hms(2026, 3, 9, 10, 23, 45)
            .unwrap()
            .checked_add_signed(chrono::Duration::milliseconds(123))
            .expect("valid datetime")
    }

    #[test]
    fn wall_clock_default_format() {
        let result = format_timestamp(
            DEFAULT_FORMAT,
            Duration::from_secs_f64(1.234),
            test_time(),
            false,
        );
        assert_eq!(result, "[10:23:45.123]");
    }

    #[test]
    fn from_zero_default_format() {
        let result = format_timestamp(
            DEFAULT_FORMAT,
            Duration::from_secs_f64(83.456),
            test_time(),
            true,
        );
        assert_eq!(result, "[00:01:23.456]");
    }

    #[test]
    fn from_zero_at_zero() {
        let result = format_timestamp(
            DEFAULT_FORMAT,
            Duration::from_secs_f64(0.0),
            test_time(),
            true,
        );
        assert_eq!(result, "[00:00:00.000]");
    }

    #[test]
    fn from_zero_over_one_hour() {
        let result = format_timestamp(
            DEFAULT_FORMAT,
            Duration::from_secs_f64(3661.234),
            test_time(),
            true,
        );
        assert_eq!(result, "[01:01:01.234]");
    }

    #[test]
    fn custom_format_seconds_only() {
        let result = format_timestamp("%S%.3f", Duration::from_secs_f64(5.678), test_time(), false);
        assert_eq!(result, "[45.123]");
    }

    #[test]
    fn custom_format_from_zero() {
        let result = format_timestamp("%M:%S", Duration::from_secs_f64(65.0), test_time(), true);
        assert_eq!(result, "[01:05]");
    }

    #[test]
    fn format_duration_large() {
        assert_eq!(
            format_duration(Duration::from_secs_f64(123.456)),
            "123.456s"
        );
    }
}
