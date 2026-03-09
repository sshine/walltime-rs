//! Process execution engine.

use chrono::Local;
use std::io::Write;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::Result;
use crate::phase::{PhaseDefinition, PhaseTracker};
use crate::summary::{PhaseTiming, RunResult};
use crate::timestamp::{TimestampFormat, format_timestamp_padded};

/// Configuration for a run.
pub struct RunConfig {
    /// The command to run.
    pub command: String,
    /// Arguments to the command.
    pub args: Vec<String>,
    /// Whether to prefix output lines with timestamps.
    pub timestamps: bool,
    /// The timestamp format to use.
    pub timestamp_format: TimestampFormat,
    /// Phase definitions for tracking.
    pub phase_definitions: Vec<PhaseDefinition>,
}

/// Write a line to the given writer, optionally prepending a timestamp.
fn write_line(
    writer: &mut dyn Write,
    line: &str,
    timestamps: bool,
    timestamp_format: TimestampFormat,
    start: Instant,
    ts_width: usize,
) -> std::io::Result<()> {
    let now = Instant::now();
    if timestamps {
        let elapsed = now.duration_since(start);
        let wall_clock = Local::now();
        let ts = format_timestamp_padded(timestamp_format, elapsed, wall_clock, ts_width);
        writeln!(writer, "{ts} {line}")?;
    } else {
        writeln!(writer, "{line}")?;
    }
    writer.flush()
}

/// Compute the current timestamp prefix width based on elapsed time.
fn current_ts_width(format: TimestampFormat, start: Instant) -> usize {
    let elapsed = Instant::now().duration_since(start);
    let wall_clock = Local::now();
    format_timestamp_padded(format, elapsed, wall_clock, 0).len()
}

/// Run a command and collect timing data.
pub async fn run(config: RunConfig) -> Result<RunResult> {
    let started_at = Local::now();
    let start = Instant::now();

    let mut child = Command::new(&config.command)
        .args(&config.args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .stdin(std::process::Stdio::inherit())
        .spawn()?;

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
    let mut phase_tracker = PhaseTracker::new(config.phase_definitions);

    // Track the current timestamp prefix width so we can right-pad for alignment.
    // Absolute format is always fixed-width, so only elapsed/both can grow.
    let mut ts_width: usize = 0;

    loop {
        tokio::select! {
            line = stdout_reader.next_line() => {
                match line? {
                    Some(line) => {
                        phase_tracker.process_line(&line, Instant::now());
                        let mut out = std::io::stdout().lock();
                        write_line(&mut out, &line, config.timestamps, config.timestamp_format, start, ts_width)?;
                        ts_width = ts_width.max(current_ts_width(config.timestamp_format, start));
                    }
                    None => {
                        // stdout closed, drain stderr
                        while let Some(line) = stderr_reader.next_line().await? {
                            phase_tracker.process_line(&line, Instant::now());
                            let mut err = std::io::stderr().lock();
                            write_line(&mut err, &line, config.timestamps, config.timestamp_format, start, ts_width)?;
                            ts_width = ts_width.max(current_ts_width(config.timestamp_format, start));
                        }
                        break;
                    }
                }
            }
            line = stderr_reader.next_line() => {
                match line? {
                    Some(line) => {
                        phase_tracker.process_line(&line, Instant::now());
                        let mut err = std::io::stderr().lock();
                        write_line(&mut err, &line, config.timestamps, config.timestamp_format, start, ts_width)?;
                        ts_width = ts_width.max(current_ts_width(config.timestamp_format, start));
                    }
                    None => {
                        // stderr closed, drain stdout
                        while let Some(line) = stdout_reader.next_line().await? {
                            phase_tracker.process_line(&line, Instant::now());
                            let mut out = std::io::stdout().lock();
                            write_line(&mut out, &line, config.timestamps, config.timestamp_format, start, ts_width)?;
                            ts_width = ts_width.max(current_ts_width(config.timestamp_format, start));
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
        exit_code: status.code(),
        started_at,
    })
}
