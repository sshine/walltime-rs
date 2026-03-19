//! Summary formatting for run results.

use chrono::{DateTime, Local};
use std::fmt::Write;
use std::time::Duration;

use terminal_size::{Width, terminal_size};

use crate::history::HistoryEntry;
use crate::timestamp::format_duration;

/// Return the terminal width, falling back to 80 columns.
fn term_width() -> usize {
    terminal_size()
        .map(|(Width(w), _)| w as usize)
        .unwrap_or(80)
}

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
///
/// `min_phase_duration` filters phases from the display only; it does not
/// affect history storage. By default the history table shows runs matching
/// the current outcome (successes on success, failures on failure).
/// `show_all` overrides this to show every run.
pub fn format_summary(
    result: &RunResult,
    history: &[HistoryEntry],
    command: &[String],
    min_phase_duration: Duration,
    show_all: bool,
) -> String {
    let mut out = String::new();

    // Filter phases for display
    let display_phases: Vec<&PhaseTiming> = result
        .phases
        .iter()
        .filter(|p| p.duration >= min_phase_duration)
        .collect();

    // Compute aggregate for hidden phases
    let hidden_phases: Vec<&PhaseTiming> = result
        .phases
        .iter()
        .filter(|p| p.duration < min_phase_duration)
        .collect();
    let has_hidden = min_phase_duration > Duration::ZERO && !hidden_phases.is_empty();

    // Determine the width of the box
    let has_phases = !display_phases.is_empty();
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

        // Pre-compute hidden aggregate values so we can include them in column widths
        let (hidden_label, hidden_sum, hidden_avg) = if has_hidden {
            let count = hidden_phases.len();
            let sum: Duration = hidden_phases.iter().map(|p| p.duration).sum();
            let avg = if count > 0 {
                sum / count as u32
            } else {
                Duration::ZERO
            };
            let label = format!("({count} phases < {})", format_duration(min_phase_duration));
            (Some(label), Some(sum), Some(avg))
        } else {
            (None, None, None)
        };

        let mut dur_w = display_phases
            .iter()
            .map(|p| format_duration(p.duration).len())
            .max()
            .unwrap_or(0);
        if let Some(sum) = hidden_sum {
            dur_w = dur_w.max(format_duration(sum).len());
        }

        // "    " (4) + name + "  " (2) + duration + "  (100.0%)" (10)
        let suffix_len = 4 + 2 + dur_w + 10;
        let name_budget = term_width().saturating_sub(suffix_len);
        let mut max_name_len = display_phases
            .iter()
            .map(|p| p.name.len())
            .max()
            .unwrap_or(0);
        if let Some(ref label) = hidden_label {
            max_name_len = max_name_len.max(label.len());
        }
        let max_name_len = max_name_len.min(name_budget);

        for phase in &display_phases {
            let pct = if total_secs > 0.0 {
                phase.duration.as_secs_f64() / total_secs * 100.0
            } else {
                0.0
            };
            let name = if phase.name.len() > max_name_len {
                format!("{}…", &phase.name[..max_name_len - 1])
            } else {
                phase.name.clone()
            };
            let _ = writeln!(
                out,
                "    {:<name_w$}  {:>dur_w$}  ({:.1}%)",
                name,
                format_duration(phase.duration),
                pct,
                name_w = max_name_len,
                dur_w = dur_w,
            );
        }
        if let (Some(label), Some(sum), Some(avg)) = (hidden_label, hidden_sum, hidden_avg) {
            let pct = if total_secs > 0.0 {
                sum.as_secs_f64() / total_secs * 100.0
            } else {
                0.0
            };
            let _ = writeln!(
                out,
                "    {:<name_w$}  {:>dur_w$}  ({:.1}%)  {} avg",
                label,
                format_duration(sum),
                pct,
                format_duration(avg),
                name_w = max_name_len,
                dur_w = dur_w,
            );
        }
    }

    if has_history {
        let current_success = result.exit_code == Some(0);
        let _ = writeln!(out);
        format_history_table(&mut out, history, command, current_success, show_all);
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

/// Format the history comparison table with exit codes.
///
/// By default, shows only runs matching the current outcome: successes when
/// the current run succeeded, failures when it failed. `show_all` overrides
/// this to include every run. An omitted-runs note is printed below the table.
fn format_history_table(
    out: &mut String,
    history: &[HistoryEntry],
    command: &[String],
    current_success: bool,
    show_all: bool,
) {
    let cmd_str = command.join(" ");

    let matches_outcome =
        |e: &&HistoryEntry| -> bool { (e.exit_code == Some(0)) == current_success };

    let entries: Vec<(usize, &HistoryEntry)> = history
        .iter()
        .enumerate()
        .filter(|(_, e)| show_all || matches_outcome(e))
        .collect();

    let omitted = history.len() - entries.len();

    if entries.is_empty() {
        return;
    }

    let _ = writeln!(out, "  History: {cmd_str} (last {} runs)", entries.len());

    // Helper to format ms as duration string
    let fmt_ms = |ms: u64| -> String { format_duration(Duration::from_millis(ms)) };

    // Column widths
    let run_w = 3.max(history.len().to_string().len());
    let date_w = 12;
    let total_w = "Total".len().max(
        entries
            .iter()
            .map(|(_, e)| fmt_ms(e.total_duration_ms).len())
            .max()
            .unwrap_or(0),
    );
    let exit_w = 4; // "Exit"

    // Top border
    let _ = write!(out, "  \u{250c}");
    let _ = write!(out, "\u{2500}{}\u{2500}", "\u{2500}".repeat(run_w));
    let _ = write!(out, "\u{252c}");
    let _ = write!(out, "\u{2500}{}\u{2500}", "\u{2500}".repeat(date_w));
    let _ = write!(out, "\u{252c}");
    let _ = write!(out, "\u{2500}{}\u{2500}", "\u{2500}".repeat(total_w));
    let _ = write!(out, "\u{252c}");
    let _ = write!(out, "\u{2500}{}\u{2500}", "\u{2500}".repeat(exit_w));
    let _ = writeln!(out, "\u{2510}");

    // Header
    let _ = write!(out, "  \u{2502}");
    let _ = write!(out, " {:>run_w$} ", "Run");
    let _ = write!(out, "\u{2502}");
    let _ = write!(out, " {:>date_w$} ", "Date");
    let _ = write!(out, "\u{2502}");
    let _ = write!(out, " {:>total_w$} ", "Total");
    let _ = write!(out, "\u{2502}");
    let _ = write!(out, " {:>exit_w$} ", "Exit");
    let _ = writeln!(out, "\u{2502}");

    // Header separator
    let _ = write!(out, "  \u{251c}");
    let _ = write!(out, "\u{2500}{}\u{2500}", "\u{2500}".repeat(run_w));
    let _ = write!(out, "\u{253c}");
    let _ = write!(out, "\u{2500}{}\u{2500}", "\u{2500}".repeat(date_w));
    let _ = write!(out, "\u{253c}");
    let _ = write!(out, "\u{2500}{}\u{2500}", "\u{2500}".repeat(total_w));
    let _ = write!(out, "\u{253c}");
    let _ = write!(out, "\u{2500}{}\u{2500}", "\u{2500}".repeat(exit_w));
    let _ = writeln!(out, "\u{2524}");

    // Data rows
    let last_original_idx = history.len() - 1;
    for (original_idx, entry) in &entries {
        let local_time: DateTime<Local> = entry.started_at.into();
        let date_str = local_time.format("%b %d %H:%M").to_string();
        let is_current = *original_idx == last_original_idx;
        let exit_str = match entry.exit_code {
            Some(code) => code.to_string(),
            None => "?".to_string(),
        };

        let _ = write!(out, "  \u{2502}");
        let _ = write!(out, " {:>run_w$} ", format!("#{}", original_idx + 1));
        let _ = write!(out, "\u{2502}");
        let _ = write!(out, " {:<date_w$} ", date_str);
        let _ = write!(out, "\u{2502}");
        let _ = write!(out, " {:>total_w$} ", fmt_ms(entry.total_duration_ms));
        let _ = write!(out, "\u{2502}");
        let _ = write!(out, " {:>exit_w$} ", exit_str);
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
    let _ = write!(out, "\u{2534}");
    let _ = write!(out, "\u{2500}{}\u{2500}", "\u{2500}".repeat(exit_w));
    let _ = writeln!(out, "\u{2518}");

    if omitted > 0 {
        let kind = if current_success {
            "failed"
        } else {
            "successful"
        };
        let noun = if omitted == 1 { "run" } else { "runs" };
        let _ = writeln!(
            out,
            "  ({omitted} {kind} {noun} omitted, use --show-all to include)"
        );
    }
}
