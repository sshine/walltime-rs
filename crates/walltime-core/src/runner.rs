//! Process execution engine.

use chrono::Local;
use std::io::Write;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::Result;
use crate::phase::{PhaseDefinition, PhaseTracker};
use crate::summary::{PhaseTiming, RunResult};
use crate::timestamp::{TimestampFormat, format_timestamp};

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
    /// Whether to set env vars that force color output in the child process.
    pub force_color: bool,
}

/// Write a line to the given writer, optionally prepending a timestamp.
fn write_line(
    writer: &mut dyn Write,
    line: &str,
    timestamps: bool,
    timestamp_format: TimestampFormat,
    start: Instant,
) -> std::io::Result<()> {
    if timestamps {
        let now = Instant::now();
        let elapsed = now.duration_since(start);
        let wall_clock = Local::now();
        let ts = format_timestamp(timestamp_format, elapsed, wall_clock);
        writeln!(writer, "{ts} {line}")?;
    } else {
        writeln!(writer, "{line}")?;
    }
    writer.flush()
}

/// Run a command and collect timing data.
pub async fn run(config: RunConfig) -> Result<RunResult> {
    let started_at = Local::now();
    let start = Instant::now();

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
    let mut phase_tracker = PhaseTracker::new(config.phase_definitions);

    loop {
        tokio::select! {
            line = stdout_reader.next_line() => {
                match line? {
                    Some(line) => {
                        phase_tracker.process_line(&line, Instant::now());
                        let mut out = std::io::stdout().lock();
                        write_line(&mut out, &line, config.timestamps, config.timestamp_format, start)?;
                    }
                    None => {
                        // stdout closed, drain stderr
                        while let Some(line) = stderr_reader.next_line().await? {
                            phase_tracker.process_line(&line, Instant::now());
                            let mut err = std::io::stderr().lock();
                            write_line(&mut err, &line, config.timestamps, config.timestamp_format, start)?;
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
                        write_line(&mut err, &line, config.timestamps, config.timestamp_format, start)?;
                    }
                    None => {
                        // stderr closed, drain stdout
                        while let Some(line) = stdout_reader.next_line().await? {
                            phase_tracker.process_line(&line, Instant::now());
                            let mut out = std::io::stdout().lock();
                            write_line(&mut out, &line, config.timestamps, config.timestamp_format, start)?;
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
