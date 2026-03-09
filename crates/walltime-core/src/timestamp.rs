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

/// Format a timestamp prefix string.
pub fn format_timestamp(
    format: TimestampFormat,
    elapsed: Duration,
    wall_clock: DateTime<Local>,
) -> String {
    match format {
        TimestampFormat::Elapsed => {
            format!("[+{}]", format_duration(elapsed))
        }
        TimestampFormat::Absolute => {
            format!("[{}]", wall_clock.format("%H:%M:%S%.3f"))
        }
        TimestampFormat::Both => {
            format!(
                "[{} +{}]",
                wall_clock.format("%H:%M:%S%.3f"),
                format_duration(elapsed)
            )
        }
    }
}

/// Format a duration as `Xs.XXXs` (e.g., `1.234s`, `0.001s`, `123.456s`).
pub fn format_duration(d: Duration) -> String {
    format!("{:.3}s", d.as_secs_f64())
}

/// Format a duration right-aligned to a given width (padding with spaces on the left).
pub fn format_duration_padded(d: Duration, width: usize) -> String {
    let s = format_duration(d);
    format!("{s:>width$}")
}

/// Format a timestamp prefix right-padded to a fixed width.
///
/// This ensures alignment when elapsed time crosses digit boundaries
/// (e.g. `[+9.147s] ` vs `[+10.123s]`).
pub fn format_timestamp_padded(
    format: TimestampFormat,
    elapsed: Duration,
    wall_clock: DateTime<Local>,
    width: usize,
) -> String {
    let ts = format_timestamp(format, elapsed, wall_clock);
    format!("{ts:<width$}")
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
        assert_eq!(result, "[+1.234s]");
    }

    #[test]
    fn elapsed_format_zero() {
        let result = format_timestamp(
            TimestampFormat::Elapsed,
            Duration::from_secs_f64(0.0),
            test_time(),
        );
        assert_eq!(result, "[+0.000s]");
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
        assert_eq!(result, "[10:23:45.123 +1.234s]");
    }

    #[test]
    fn format_duration_large() {
        assert_eq!(
            format_duration(Duration::from_secs_f64(123.456)),
            "123.456s"
        );
    }
}
