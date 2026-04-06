//! CLI argument definitions using [`clap`].
//!
//! The [`Args`] struct derives [`Parser`] and describes every flag and option
//! accepted by the `wtime` binary. See the crate-level documentation for the
//! full help output.

use clap::Parser;
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

    /// Regex with a capture group for dynamic phase names (repeatable).
    #[arg(short = 'd', long = "dynamic-phase", value_name = "REGEX")]
    pub dynamic_phases: Vec<String>,

    /// Hide phases shorter than this threshold in the summary (seconds).
    #[arg(
        short = 'm',
        long = "min-phase",
        value_name = "SECONDS",
        default_value = "0"
    )]
    pub min_phase: f64,

    /// Suppress the timing summary.
    #[arg(long = "no-summary")]
    pub no_summary: bool,

    /// Don't save or show the run log.
    #[arg(long = "no-log")]
    pub no_log: bool,

    /// Log file path.
    #[arg(long = "log-file", default_value = ".walltime.jsonl")]
    pub log_file: String,

    /// Show all runs in history (by default, only matching-outcome runs are shown).
    #[arg(short = 'a', long = "show-all")]
    pub show_all: bool,

    /// Command to run.
    #[arg(trailing_var_arg = true, required = true)]
    pub command: Vec<String>,
}
