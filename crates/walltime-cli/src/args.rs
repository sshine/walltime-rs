//! CLI argument definitions.

use clap::{Parser, ValueEnum};
use walltime_core::timestamp::DEFAULT_FORMAT;

/// A modern replacement for the UNIX `time` command.
///
/// Runs a command and provides a colorful timing summary, optional line
/// timestamps, phase tracking, and run history comparison.
#[derive(Debug, Parser)]
#[command(name = "wtime", version, about)]
pub struct Args {
    /// Enable line timestamp prefixing.
    #[arg(short = 't', long = "timestamps")]
    pub timestamps: bool,

    /// Timestamp format (chrono syntax).
    #[arg(short = 'f', long = "timestamp-format", default_value = DEFAULT_FORMAT)]
    pub timestamp_format: String,

    /// Count timestamps from 00:00:00.000 instead of wall-clock time.
    #[arg(short = '0', long = "from-zero")]
    pub from_zero: bool,

    /// Define a phase boundary (repeatable). Format: NAME=REGEX
    #[arg(short = 'p', long = "phase", value_name = "NAME=REGEX")]
    pub phases: Vec<String>,

    /// Suppress the timing summary.
    #[arg(long = "no-summary")]
    pub no_summary: bool,

    /// Don't save or show the run log.
    #[arg(long = "no-log")]
    pub no_log: bool,

    /// Log file path.
    #[arg(long = "log-file", default_value = ".walltime.jsonl")]
    pub log_file: String,

    /// When to use colors.
    #[arg(long = "color", default_value = "auto")]
    pub color: ColorChoice,

    /// Command to run.
    #[arg(trailing_var_arg = true, required = true)]
    pub command: Vec<String>,
}

/// Color output choice for CLI.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ColorChoice {
    /// Automatically detect terminal support.
    Auto,
    /// Always use colors.
    Always,
    /// Never use colors.
    Never,
}
