//! Summary formatting for run results.

use chrono::{DateTime, Local};
use std::fmt::Write;
use std::time::Duration;

use crate::history::HistoryEntry;
use crate::timestamp::format_duration;

/// Result of a completed run.
#[derive(Debug)]
pub struct RunResult {
    /// Total duration of the run.
    pub total: Duration,
    /// Per-phase timing data.
    pub phases: Vec<PhaseTiming>,
    /// Exit code of the child process.
    pub exit_code: Option<i32>,
    /// When the run started (wall-clock).
    pub started_at: DateTime<Local>,
}

/// Timing for a single phase.
#[derive(Debug, Clone)]
pub struct PhaseTiming {
    /// Name of the phase.
    pub name: String,
    /// Duration of the phase.
    pub duration: Duration,
}

/// Format the summary block (without colors).
pub fn format_summary(result: &RunResult, history: &[HistoryEntry], command: &[String]) -> String {
    let mut out = String::new();

    // Determine the width of the box
    let has_phases = !result.phases.is_empty();
    let has_history = history.len() > 1;

    let min_width = if has_history { 50 } else { 30 };

    let rule = "\u{2500}".repeat(min_width);

    let _ = writeln!(out, "\n{rule}");
    let _ = writeln!(out, "  walltime summary");
    let _ = writeln!(out, "{rule}");
    let _ = writeln!(out, "  Total:      {}", format_duration(result.total));

    if has_phases {
        let _ = writeln!(out);
        let _ = writeln!(out, "  Phases:");
        let total_secs = result.total.as_secs_f64();
        let max_name_len = result
            .phases
            .iter()
            .map(|p| p.name.len())
            .max()
            .unwrap_or(0);
        for phase in &result.phases {
            let pct = if total_secs > 0.0 {
                phase.duration.as_secs_f64() / total_secs * 100.0
            } else {
                0.0
            };
            let _ = writeln!(
                out,
                "    {:<width$}  {}  ({:.1}%)",
                phase.name,
                format_duration(phase.duration),
                pct,
                width = max_name_len,
            );
        }
    }

    if has_history {
        let _ = writeln!(out);
        format_history_table(&mut out, history, command);
    }

    let _ = writeln!(out);
    let exit_str = match result.exit_code {
        Some(code) => code.to_string(),
        None => "unknown".to_string(),
    };
    let _ = writeln!(out, "  Exit code:  {exit_str}");
    let _ = writeln!(out, "{rule}");

    out
}

/// Format the history comparison table.
fn format_history_table(out: &mut String, history: &[HistoryEntry], command: &[String]) {
    let cmd_str = command.join(" ");
    let _ = writeln!(out, "  History: {cmd_str} (last {} runs)", history.len());

    // Collect all unique phase names in order
    let mut phase_names: Vec<String> = Vec::new();
    for entry in history {
        for phase in &entry.phases {
            if !phase_names.contains(&phase.name) {
                phase_names.push(phase.name.clone());
            }
        }
    }

    // Helper to format ms as duration string
    let fmt_ms = |ms: u64| -> String { format_duration(Duration::from_millis(ms)) };

    // Column widths - compute from actual data so the table never overflows
    let run_w = 3.max(history.len().to_string().len());
    let date_w = 12;
    let total_w = "Total".len().max(
        history
            .iter()
            .map(|e| fmt_ms(e.total_duration_ms).len())
            .max()
            .unwrap_or(0),
    );
    let phase_ws: Vec<usize> = phase_names
        .iter()
        .map(|name| {
            let header_w = name.len();
            let data_w = history
                .iter()
                .map(|e| {
                    e.phases
                        .iter()
                        .find(|p| &p.name == name)
                        .map(|p| fmt_ms(p.duration_ms).len())
                        .unwrap_or(0)
                })
                .max()
                .unwrap_or(0);
            header_w.max(data_w)
        })
        .collect();

    // Top border
    let _ = write!(out, "  \u{250c}");
    let _ = write!(out, "\u{2500}{}\u{2500}", "\u{2500}".repeat(run_w));
    let _ = write!(out, "\u{252c}");
    let _ = write!(out, "\u{2500}{}\u{2500}", "\u{2500}".repeat(date_w));
    let _ = write!(out, "\u{252c}");
    let _ = write!(out, "\u{2500}{}\u{2500}", "\u{2500}".repeat(total_w));
    for w in &phase_ws {
        let _ = write!(out, "\u{252c}");
        let _ = write!(out, "\u{2500}{}\u{2500}", "\u{2500}".repeat(*w));
    }
    let _ = writeln!(out, "\u{2510}");

    // Header
    let _ = write!(out, "  \u{2502}");
    let _ = write!(out, " {:>run_w$} ", "Run");
    let _ = write!(out, "\u{2502}");
    let _ = write!(out, " {:>date_w$} ", "Date");
    let _ = write!(out, "\u{2502}");
    let _ = write!(out, " {:>total_w$} ", "Total");
    for (i, name) in phase_names.iter().enumerate() {
        let _ = write!(out, "\u{2502}");
        let _ = write!(out, " {:>width$} ", name, width = phase_ws[i]);
    }
    let _ = writeln!(out, "\u{2502}");

    // Header separator
    let _ = write!(out, "  \u{251c}");
    let _ = write!(out, "\u{2500}{}\u{2500}", "\u{2500}".repeat(run_w));
    let _ = write!(out, "\u{253c}");
    let _ = write!(out, "\u{2500}{}\u{2500}", "\u{2500}".repeat(date_w));
    let _ = write!(out, "\u{253c}");
    let _ = write!(out, "\u{2500}{}\u{2500}", "\u{2500}".repeat(total_w));
    for w in &phase_ws {
        let _ = write!(out, "\u{253c}");
        let _ = write!(out, "\u{2500}{}\u{2500}", "\u{2500}".repeat(*w));
    }
    let _ = writeln!(out, "\u{2524}");

    // Data rows
    let last_idx = history.len() - 1;
    for (i, entry) in history.iter().enumerate() {
        let local_time: DateTime<Local> = entry.started_at.into();
        let date_str = local_time.format("%b %d %H:%M").to_string();
        let is_current = i == last_idx;

        let _ = write!(out, "  \u{2502}");
        let _ = write!(out, " {:>run_w$} ", format!("#{}", i + 1));
        let _ = write!(out, "\u{2502}");
        let _ = write!(out, " {:<date_w$} ", date_str);
        let _ = write!(out, "\u{2502}");
        let _ = write!(out, " {:>total_w$} ", fmt_ms(entry.total_duration_ms));
        for (j, name) in phase_names.iter().enumerate() {
            let _ = write!(out, "\u{2502}");
            let phase_ms = entry
                .phases
                .iter()
                .find(|p| &p.name == name)
                .map(|p| p.duration_ms)
                .unwrap_or(0);
            let _ = write!(out, " {:>width$} ", fmt_ms(phase_ms), width = phase_ws[j]);
        }
        let _ = write!(out, "\u{2502}");
        if is_current {
            let _ = write!(out, " \u{2190} current");
        }
        let _ = writeln!(out);
    }

    // Bottom border
    let _ = write!(out, "  \u{2514}");
    let _ = write!(out, "\u{2500}{}\u{2500}", "\u{2500}".repeat(run_w));
    let _ = write!(out, "\u{2534}");
    let _ = write!(out, "\u{2500}{}\u{2500}", "\u{2500}".repeat(date_w));
    let _ = write!(out, "\u{2534}");
    let _ = write!(out, "\u{2500}{}\u{2500}", "\u{2500}".repeat(total_w));
    for w in &phase_ws {
        let _ = write!(out, "\u{2534}");
        let _ = write!(out, "\u{2500}{}\u{2500}", "\u{2500}".repeat(*w));
    }
    let _ = writeln!(out, "\u{2518}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::{HistoryEntry, PhaseTime};
    use chrono::TimeZone;

    fn make_result(total_ms: u64, phases: Vec<(&str, u64)>, exit_code: i32) -> RunResult {
        let total_secs = total_ms as f64 / 1000.0;
        RunResult {
            total: Duration::from_secs_f64(total_secs),
            phases: phases
                .into_iter()
                .map(|(name, ms)| PhaseTiming {
                    name: name.to_string(),
                    duration: Duration::from_millis(ms),
                })
                .collect(),
            exit_code: Some(exit_code),
            started_at: Local.with_ymd_and_hms(2026, 3, 9, 10, 45, 0).unwrap(),
        }
    }

    #[test]
    fn summary_simple() {
        let result = make_result(2000, vec![], 0);
        let output = format_summary(&result, &[], &["sleep".into(), "2".into()]);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn summary_with_phases() {
        let result = make_result(2000, vec![("compile", 1456), ("link", 544)], 0);
        let output = format_summary(&result, &[], &["cargo".into(), "build".into()]);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn summary_nonzero_exit() {
        let result = make_result(500, vec![], 1);
        let output = format_summary(&result, &[], &["false".into()]);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn summary_with_history() {
        let result = make_result(1800, vec![("compile", 1200), ("link", 600)], 0);
        let history = vec![
            HistoryEntry {
                command: vec!["cargo".into(), "build".into()],
                started_at: "2026-03-09T09:23:00Z".parse().expect("valid"),
                total_duration_ms: 2345,
                phases: vec![
                    PhaseTime {
                        name: "compile".into(),
                        duration_ms: 1456,
                    },
                    PhaseTime {
                        name: "link".into(),
                        duration_ms: 544,
                    },
                ],
                exit_code: Some(0),
            },
            HistoryEntry {
                command: vec!["cargo".into(), "build".into()],
                started_at: "2026-03-09T09:30:00Z".parse().expect("valid"),
                total_duration_ms: 1890,
                phases: vec![
                    PhaseTime {
                        name: "compile".into(),
                        duration_ms: 1200,
                    },
                    PhaseTime {
                        name: "link".into(),
                        duration_ms: 390,
                    },
                ],
                exit_code: Some(0),
            },
            HistoryEntry {
                command: vec!["cargo".into(), "build".into()],
                started_at: "2026-03-09T09:45:00Z".parse().expect("valid"),
                total_duration_ms: 1800,
                phases: vec![
                    PhaseTime {
                        name: "compile".into(),
                        duration_ms: 1200,
                    },
                    PhaseTime {
                        name: "link".into(),
                        duration_ms: 600,
                    },
                ],
                exit_code: Some(0),
            },
        ];
        let output = format_summary(&result, &history, &["cargo".into(), "build".into()]);
        // Redact dates since they depend on local timezone
        let output = regex::Regex::new(r"[A-Z][a-z]{2} \d{2} \d{2}:\d{2}")
            .expect("valid regex")
            .replace_all(&output, "[DATE]")
            .to_string();
        insta::assert_snapshot!(output);
    }

    #[test]
    fn summary_with_history_wide_total() {
        let result = make_result(16705, vec![], 0);
        let history = vec![
            HistoryEntry {
                command: vec!["cargo".into(), "build".into()],
                started_at: "2026-03-09T09:23:00Z".parse().expect("valid"),
                total_duration_ms: 83,
                phases: vec![],
                exit_code: Some(0),
            },
            HistoryEntry {
                command: vec!["cargo".into(), "build".into()],
                started_at: "2026-03-09T09:30:00Z".parse().expect("valid"),
                total_duration_ms: 84,
                phases: vec![],
                exit_code: Some(0),
            },
            HistoryEntry {
                command: vec!["cargo".into(), "build".into()],
                started_at: "2026-03-09T09:45:00Z".parse().expect("valid"),
                total_duration_ms: 16705,
                phases: vec![],
                exit_code: Some(0),
            },
        ];
        let output = format_summary(&result, &history, &["cargo".into(), "build".into()]);
        let output = regex::Regex::new(r"[A-Z][a-z]{2} \d{2} \d{2}:\d{2}")
            .expect("valid regex")
            .replace_all(&output, "[DATE]")
            .to_string();
        insta::assert_snapshot!(output);
    }
}
