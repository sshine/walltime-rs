//! Timestamp formatting for output line prefixes.

use chrono::{DateTime, Local};
use std::fmt;
use std::time::Duration;

/// Format for timestamp prefixes on output lines.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TimestampFormat {
    /// Show elapsed time since start: `[+1.234s]`
    #[default]
    Elapsed,
    /// Show wall-clock time: `[10:23:45.123]`
    Absolute,
    /// Show both: `[10:23:45.123 +1.234s]`
    Both,
}

impl fmt::Display for TimestampFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Elapsed => write!(f, "elapsed"),
            Self::Absolute => write!(f, "absolute"),
            Self::Both => write!(f, "both"),
        }
    }
}

/// Width of the elapsed duration field inside timestamp prefixes.
///
/// Right-aligned to 9 chars, enough for 4-digit seconds (`XXXX.XXXs`),
/// so the prefix width stays constant and output doesn't shift.
const ELAPSED_DURATION_WIDTH: usize = 9;

/// Format a timestamp prefix string.
///
/// The elapsed duration is right-aligned to a fixed width so that the
/// prefix is always the same number of characters regardless of elapsed time.
pub fn format_timestamp(
    format: TimestampFormat,
    elapsed: Duration,
    wall_clock: DateTime<Local>,
) -> String {
    let dur = format_duration(elapsed);
    match format {
        TimestampFormat::Elapsed => {
            format!("[+{dur:>ELAPSED_DURATION_WIDTH$}]")
        }
        TimestampFormat::Absolute => {
            format!("[{}]", wall_clock.format("%H:%M:%S%.3f"))
        }
        TimestampFormat::Both => {
            format!(
                "[{} +{dur:>ELAPSED_DURATION_WIDTH$}]",
                wall_clock.format("%H:%M:%S%.3f"),
            )
        }
    }
}

/// Format a duration as `Xs.XXXs` (e.g., `1.234s`, `0.001s`, `123.456s`).
pub fn format_duration(d: Duration) -> String {
    format!("{:.3}s", d.as_secs_f64())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn test_time() -> DateTime<Local> {
        // Create a fixed local time for testing
        Local
            .with_ymd_and_hms(2026, 3, 9, 10, 23, 45)
            .unwrap()
            .checked_add_signed(chrono::Duration::milliseconds(123))
            .expect("valid datetime")
    }

    #[test]
    fn elapsed_format() {
        let result = format_timestamp(
            TimestampFormat::Elapsed,
            Duration::from_secs_f64(1.234),
            test_time(),
        );
        assert_eq!(result, "[+   1.234s]");
    }

    #[test]
    fn elapsed_format_zero() {
        let result = format_timestamp(
            TimestampFormat::Elapsed,
            Duration::from_secs_f64(0.0),
            test_time(),
        );
        assert_eq!(result, "[+   0.000s]");
    }

    #[test]
    fn elapsed_format_large() {
        let result = format_timestamp(
            TimestampFormat::Elapsed,
            Duration::from_secs_f64(1234.567),
            test_time(),
        );
        assert_eq!(result, "[+1234.567s]");
    }

    #[test]
    fn elapsed_format_double_digits() {
        // Verify alignment is consistent across digit boundaries
        let r1 = format_timestamp(
            TimestampFormat::Elapsed,
            Duration::from_secs_f64(9.147),
            test_time(),
        );
        let r2 = format_timestamp(
            TimestampFormat::Elapsed,
            Duration::from_secs_f64(10.123),
            test_time(),
        );
        assert_eq!(r1.len(), r2.len());
        assert_eq!(r1, "[+   9.147s]");
        assert_eq!(r2, "[+  10.123s]");
    }

    #[test]
    fn absolute_format() {
        let result = format_timestamp(
            TimestampFormat::Absolute,
            Duration::from_secs_f64(1.234),
            test_time(),
        );
        assert_eq!(result, "[10:23:45.123]");
    }

    #[test]
    fn both_format() {
        let result = format_timestamp(
            TimestampFormat::Both,
            Duration::from_secs_f64(1.234),
            test_time(),
        );
        assert_eq!(result, "[10:23:45.123 +   1.234s]");
    }

    #[test]
    fn format_duration_large() {
        assert_eq!(
            format_duration(Duration::from_secs_f64(123.456)),
            "123.456s"
        );
    }
}
