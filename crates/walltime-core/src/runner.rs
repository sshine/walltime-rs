//! Process execution engine.

use chrono::Local;
use owo_colors::OwoColorize;
use std::io::Write;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::Result;
use crate::phase::{DynamicPhaseDefinition, PhaseDefinition, PhaseTracker};
use crate::summary::{PhaseTiming, RunResult};
use crate::timestamp::format_timestamp;

/// Configuration for a run.
pub struct RunConfig {
    /// The command to run.
    pub command: String,
    /// Arguments to the command.
    pub args: Vec<String>,
    /// Whether to prefix output lines with timestamps.
    pub timestamps: bool,
    /// The chrono format string for timestamps.
    pub timestamp_format: String,
    /// Whether to count from zero instead of wall-clock time.
    pub from_zero: bool,
    /// Phase definitions for tracking.
    pub phase_definitions: Vec<PhaseDefinition>,
    /// Dynamic phase definitions for tracking.
    pub dynamic_phase_definitions: Vec<DynamicPhaseDefinition>,
    /// Whether to set env vars that force color output in the child process.
    pub force_color: bool,
    /// Minimum phase duration threshold for timestamp coloring.
    pub min_phase_duration: Duration,
    /// Whether to color walltime's own output (timestamps).
    pub color_output: bool,
}

/// Timestamp and coloring settings extracted from [`RunConfig`].
struct LineConfig {
    timestamps: bool,
    timestamp_format: String,
    from_zero: bool,
    min_phase_duration: Duration,
    color_output: bool,
}

/// Mutable state for tracking inter-line timing.
struct LineState {
    /// When the process started.
    start: Instant,
    /// When the last output line was written.
    last_line_time: Option<Instant>,
}

/// Write a line to the given writer, optionally prepending a timestamp.
///
/// When `color_output` is true on the config, brackets are bold and the time
/// is green (or yellow when the delta since the previous line exceeds
/// `min_phase_duration`).
fn write_line(
    writer: &mut dyn Write,
    line: &str,
    config: &LineConfig,
    state: &mut LineState,
) -> std::io::Result<()> {
    if config.timestamps {
        let now = Instant::now();
        let elapsed = now.duration_since(state.start);
        let wall_clock = Local::now();
        let formatted = format_timestamp(
            &config.timestamp_format,
            elapsed,
            wall_clock,
            config.from_zero,
        );
        // formatted is "[HH:MM:SS.mmm]" – strip the brackets for coloring
        let inner = &formatted[1..formatted.len() - 1];

        if config.color_output {
            let delta_exceeds = state
                .last_line_time
                .map(|prev| now.duration_since(prev) >= config.min_phase_duration)
                .unwrap_or(false)
                && config.min_phase_duration > Duration::ZERO;

            if delta_exceeds {
                write!(writer, "{}{}{} ", "[".bold(), inner.yellow(), "]".bold())?;
            } else {
                write!(writer, "{}{}{} ", "[".bold(), inner.green(), "]".bold())?;
            }
            writeln!(writer, "{line}")?;
        } else {
            writeln!(writer, "{formatted} {line}")?;
        }
        state.last_line_time = Some(now);
    } else {
        writeln!(writer, "{line}")?;
    }
    writer.flush()
}

/// Run a command and collect timing data.
pub async fn run(config: RunConfig) -> Result<RunResult> {
    let started_at = Local::now();
    let start = Instant::now();

    let line_config = LineConfig {
        timestamps: config.timestamps,
        timestamp_format: config.timestamp_format.clone(),
        from_zero: config.from_zero,
        min_phase_duration: config.min_phase_duration,
        color_output: config.color_output,
    };

    let mut cmd = Command::new(&config.command);
    cmd.args(&config.args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .stdin(std::process::Stdio::inherit());

    if config.force_color {
        cmd.env("CLICOLOR_FORCE", "1")
            .env("FORCE_COLOR", "1")
            .env("CARGO_TERM_COLOR", "always")
            .env("GCC_COLORS", "1");
    }

    let mut child = cmd.spawn()?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| crate::Error::Other("failed to capture stdout".to_string()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| crate::Error::Other("failed to capture stderr".to_string()))?;

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();
    let mut phase_tracker =
        PhaseTracker::new(config.phase_definitions, config.dynamic_phase_definitions);

    let mut line_state = LineState {
        start,
        last_line_time: None,
    };

    loop {
        tokio::select! {
            line = stdout_reader.next_line() => {
                match line? {
                    Some(line) => {
                        let stripped = strip_ansi_escapes::strip_str(&line);
                        phase_tracker.process_line(&stripped, Instant::now());
                        let mut out = std::io::stdout().lock();
                        write_line(&mut out, &line, &line_config, &mut line_state)?;
                    }
                    None => {
                        // stdout closed, drain stderr
                        while let Some(line) = stderr_reader.next_line().await? {
                            let stripped = strip_ansi_escapes::strip_str(&line);
                            phase_tracker.process_line(&stripped, Instant::now());
                            let mut err = std::io::stderr().lock();
                            write_line(&mut err, &line, &line_config, &mut line_state)?;
                        }
                        break;
                    }
                }
            }
            line = stderr_reader.next_line() => {
                match line? {
                    Some(line) => {
                        let stripped = strip_ansi_escapes::strip_str(&line);
                        phase_tracker.process_line(&stripped, Instant::now());
                        let mut err = std::io::stderr().lock();
                        write_line(&mut err, &line, &line_config, &mut line_state)?;
                    }
                    None => {
                        // stderr closed, drain stdout
                        while let Some(line) = stdout_reader.next_line().await? {
                            let stripped = strip_ansi_escapes::strip_str(&line);
                            phase_tracker.process_line(&stripped, Instant::now());
                            let mut out = std::io::stdout().lock();
                            write_line(&mut out, &line, &line_config, &mut line_state)?;
                        }
                        break;
                    }
                }
            }
        }
    }

    let status = child.wait().await?;
    let end = Instant::now();
    let total = end.duration_since(start);

    phase_tracker.finish(end);

    let phases = phase_tracker
        .records()
        .iter()
        .map(|r| PhaseTiming {
            name: r.name.clone(),
            duration: r.duration,
        })
        .collect();

    Ok(RunResult {
        total,
        phases,
        exit_code: status.code().or_else(|| {
            #[cfg(unix)]
            {
                use std::os::unix::process::ExitStatusExt;
                status.signal().map(|sig| 128 + sig)
            }
            #[cfg(not(unix))]
            {
                None
            }
        }),
        started_at,
    })
}
