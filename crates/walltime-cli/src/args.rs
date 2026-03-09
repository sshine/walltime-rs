//! CLI argument definitions.

use clap::{Parser, ValueEnum};

/// A modern replacement for the UNIX `time` command.
///
/// Runs a command and provides a colorful timing summary, optional line
/// timestamps, phase tracking, and run history comparison.
#[derive(Debug, Parser)]
#[command(name = "walltime", version, about)]
pub struct Args {
    /// Enable line timestamp prefixing.
    #[arg(short = 't', long = "timestamps")]
    pub timestamps: bool,

    /// Timestamp format: elapsed, absolute, or both.
    #[arg(long = "timestamp-format", default_value = "elapsed")]
    pub timestamp_format: TimestampFormatArg,

    /// Define a phase boundary (repeatable). Format: NAME=REGEX
    #[arg(short = 'p', long = "phase", value_name = "NAME=REGEX")]
    pub phases: Vec<String>,

    /// Suppress the timing summary.
    #[arg(long = "no-summary")]
    pub no_summary: bool,

    /// Don't save or show history.
    #[arg(long = "no-history")]
    pub no_history: bool,

    /// History file path.
    #[arg(long = "log-file", default_value = ".walltime.jsonl")]
    pub log_file: String,

    /// When to use colors: auto, always, never.
    #[arg(long = "color", default_value = "auto")]
    pub color: ColorChoice,

    /// Command to run.
    #[arg(trailing_var_arg = true, required = true)]
    pub command: Vec<String>,
}

/// Timestamp format choice for CLI.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TimestampFormatArg {
    /// Elapsed time since start.
    Elapsed,
    /// Wall-clock time.
    Absolute,
    /// Both elapsed and wall-clock.
    Both,
}

impl From<TimestampFormatArg> for walltime_core::timestamp::TimestampFormat {
    fn from(arg: TimestampFormatArg) -> Self {
        match arg {
            TimestampFormatArg::Elapsed => Self::Elapsed,
            TimestampFormatArg::Absolute => Self::Absolute,
            TimestampFormatArg::Both => Self::Both,
        }
    }
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
