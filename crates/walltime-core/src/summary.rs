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
/// affect history storage.
pub fn format_summary(
    result: &RunResult,
    history: &[HistoryEntry],
    command: &[String],
    min_phase_duration: Duration,
) -> String {
    let mut out = String::new();

    // Filter phases for display
    let display_phases: Vec<&PhaseTiming> = result
        .phases
        .iter()
        .filter(|p| p.duration >= min_phase_duration)
        .collect();

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
        let dur_w = display_phases
            .iter()
            .map(|p| format_duration(p.duration).len())
            .max()
            .unwrap_or(0);
        // "    " (4) + name + "  " (2) + duration + "  (100.0%)" (10)
        let suffix_len = 4 + 2 + dur_w + 10;
        let name_budget = term_width().saturating_sub(suffix_len);
        let max_name_len = display_phases
            .iter()
            .map(|p| p.name.len())
            .max()
            .unwrap_or(0)
            .min(name_budget);
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
                "    {:<width$}  {}  ({:.1}%)",
                name,
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

/// Format the history comparison table (total time only, no per-phase breakdown).
fn format_history_table(out: &mut String, history: &[HistoryEntry], command: &[String]) {
    let cmd_str = command.join(" ");
    let _ = writeln!(out, "  History: {cmd_str} (last {} runs)", history.len());

    // Helper to format ms as duration string
    let fmt_ms = |ms: u64| -> String { format_duration(Duration::from_millis(ms)) };

    // Column widths
    let run_w = 3.max(history.len().to_string().len());
    let date_w = 12;
    let total_w = "Total".len().max(
        history
            .iter()
            .map(|e| fmt_ms(e.total_duration_ms).len())
            .max()
            .unwrap_or(0),
    );

    // Top border
    let _ = write!(out, "  \u{250c}");
    let _ = write!(out, "\u{2500}{}\u{2500}", "\u{2500}".repeat(run_w));
    let _ = write!(out, "\u{252c}");
    let _ = write!(out, "\u{2500}{}\u{2500}", "\u{2500}".repeat(date_w));
    let _ = write!(out, "\u{252c}");
    let _ = write!(out, "\u{2500}{}\u{2500}", "\u{2500}".repeat(total_w));
    let _ = writeln!(out, "\u{2510}");

    // Header
    let _ = write!(out, "  \u{2502}");
    let _ = write!(out, " {:>run_w$} ", "Run");
    let _ = write!(out, "\u{2502}");
    let _ = write!(out, " {:>date_w$} ", "Date");
    let _ = write!(out, "\u{2502}");
    let _ = write!(out, " {:>total_w$} ", "Total");
    let _ = writeln!(out, "\u{2502}");

    // Header separator
    let _ = write!(out, "  \u{251c}");
    let _ = write!(out, "\u{2500}{}\u{2500}", "\u{2500}".repeat(run_w));
    let _ = write!(out, "\u{253c}");
    let _ = write!(out, "\u{2500}{}\u{2500}", "\u{2500}".repeat(date_w));
    let _ = write!(out, "\u{253c}");
    let _ = write!(out, "\u{2500}{}\u{2500}", "\u{2500}".repeat(total_w));
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
    let _ = writeln!(out, "\u{2518}");
}
