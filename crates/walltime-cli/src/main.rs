//! # CLI for measuring time spent in a process.
//!
//! `wtime` is a modern replacement for the UNIX `time` command. It runs a
//! command, streams its output (optionally with timestamps), tracks phases
//! via regex matching, and prints a colorful timing summary with run history
//! comparison.
//!
//! # Examples
//!
//! ## Default
//!
//! Without any parameters, `wtime` produces a summary with history at the end:
//!
//! ```text
//! $ wtime cargo build
//!    Compiling proc-macro2 v1.0.106
//!    ...
//!    Compiling walltime-cli v0.1.0
//!     Finished `dev` profile in 7.31s
//!
//! ──────────────────────────────────────────────────
//!   walltime summary
//! ──────────────────────────────────────────────────
//!   Total:      7.329s
//!
//!   History: cargo build (last 6 runs)
//!   ┌─────┬──────────────┬────────┬──────┐
//!   │ Run │         Date │  Total │ Exit │
//!   ├─────┼──────────────┼────────┼──────┤
//!   │  #1 │ Apr 06 14:42 │ 7.615s │    0 │
//!   ...
//!   │  #6 │ Apr 06 14:46 │ 7.328s │    0 │ ← current
//!   └─────┴──────────────┴────────┴──────┘
//!
//!   Exit code:  0
//! ──────────────────────────────────────────────────
//! ```
//!
//! Only successful runs (exit code 0) are included by default.
//!
//! When the program fails it prints all the failed runs instead.
//!
//! Use `wtime -a ...` to include all runs regardless of exit code.
//!
//! ## Timestamps
//!
//! To add wall-clock timestamps to every output line:
//!
//! ```text
//! $ wtime -t cargo build
//! [14:47:46.143]    Compiling proc-macro2 v1.0.106
//! [14:47:46.143]    Compiling unicode-ident v1.0.24
//! ...
//! [14:47:52.923]     Finished `dev` profile in 6.85s
//! ...
//! ```
//!
//! Timestamp format can be modified with `-f <format>`, e.g. UNIX epoch with milliseconds:
//!
//! ```text
//! $ wtime -t -0 -f '%s%.3f' cargo build
//! [946684800.090]    Compiling proc-macro2 v1.0.106
//! ...
//! [946684801.881]    Compiling errno v0.3.14
//! ...
//! [946684807.345]     Finished `dev` profile in 7.33s
//! ...
//! ```
//!
//! ## Zero timestamps
//!
//! To add timestamps that begin from zero instead of current local time:
//!
//! ```text
//! $ wtime -t -0 cargo build
//! [00:00:00.086]    Compiling proc-macro2 v1.0.106
//! [00:00:00.086]    Compiling unicode-ident v1.0.24
//! ...
//! [00:00:07.018]     Finished `dev` profile in 7.01s
//! ...
//! ```
//!
//! ## Dynamic phase-tracking
//!
//! To track phases of a process, use `-d` and a regex with a capture group:
//!
//! ```text
//! $ wtime -t -0 -d 'Compiling ([^ ]*)' cargo build
//! ...
//!
//!   Phases:
//!     proc-macro2           0.000s  (0.0%)
//!     ...
//!     regex-automata        1.119s  (15.7%)
//!     ...
//!     chrono                0.538s  (7.5%)
//!     walltime-core         0.480s  (6.7%)
//!     walltime-cli          0.500s  (7.0%)
//! ...
//! ```
//!
//! This helps track what part of a very verbose output is taking a long time.
//!
//! Before matching the regex, all ANSI codes are stripped from the string.
//!
//! To hide phases that take shorter than some amount of seconds, use `-m N` to focus on the slow ones:
//!
//! ```text
//! $ wtime -t -0 -d 'Compiling ([^ ]*)' -m 1 cargo build
//! ...
//!
//!   Phases:
//!     ctrlc                 1.460s  (19.7%)
//!     clap                  1.017s  (13.7%)
//!     (57 phases < 1.000s)  4.838s  (65.4%)  0.085s avg
//! ...
//! ```
//!
//! Using `-m N` only omits phases when printing, all phases are still measured and saved.
//!
//! ## Full help output
//!
//! ```text
//! A modern replacement for the UNIX `time` command.
//!
//! Runs a command and provides a colorful timing summary, optional line timestamps, phase tracking, and run history comparison.
//!
//! Usage: wtime [OPTIONS] <COMMAND>...
//!
//! Arguments:
//!   <COMMAND>...
//!           Command to run
//!
//! Options:
//!   -t, --timestamps
//!           Enable line timestamp prefixing
//!
//!   -f, --timestamp-format <TIMESTAMP_FORMAT>
//!           Timestamp format (chrono syntax)
//!           
//!           [default: %H:%M:%S%.3f]
//!
//!   -0, --from-zero
//!           Count timestamps from 00:00:00.000 instead of wall-clock time
//!
//!   -p, --phase <NAME=REGEX>
//!           Define a phase boundary (repeatable). Format: NAME=REGEX
//!
//!   -d, --dynamic-phase <REGEX>
//!           Regex with a capture group for dynamic phase names (repeatable)
//!
//!   -m, --min-phase <SECONDS>
//!           Hide phases shorter than this threshold in the summary (seconds)
//!           
//!           [default: 0]
//!
//!       --no-summary
//!           Suppress the timing summary
//!
//!       --no-log
//!           Don't save or show the run log
//!
//!       --log-file <LOG_FILE>
//!           Log file path
//!           
//!           [default: .walltime.jsonl]
//!
//!   -a, --show-all
//!           Show all runs in history (by default, only matching-outcome runs are shown)
//!
//!   -h, --help
//!           Print help (see a summary with '-h')
//!
//!   -V, --version
//!           Print version
//! ```

mod args;

use std::path::Path;
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};

use clap::Parser;
use owo_colors::OwoColorize;

use walltime_core::history::{self, HistoryEntry, PhaseTime};
use walltime_core::phase::{DynamicPhaseDefinition, PhaseDefinition};
use walltime_core::runner::{self, RunConfig};
use walltime_core::summary;

use args::Args;

#[tokio::main]
async fn main() -> ExitCode {
    // Intercept Ctrl+C so the child process dies but the parent survives to print the summary.
    // Second Ctrl+C force-exits with code 130.
    static INTERRUPTED: AtomicBool = AtomicBool::new(false);
    ctrlc::set_handler(move || {
        if INTERRUPTED.swap(true, Ordering::SeqCst) {
            std::process::exit(130);
        }
    })
    .expect("failed to set Ctrl+C handler");

    let args = Args::parse();

    // Parse phase definitions
    let phase_definitions: Vec<PhaseDefinition> = match args
        .phases
        .iter()
        .map(|s| PhaseDefinition::parse(s))
        .collect::<walltime_core::Result<Vec<_>>>()
    {
        Ok(defs) => defs,
        Err(e) => {
            eprintln!("{}: {e}", "error".red().bold());
            return ExitCode::from(2);
        }
    };

    // Parse dynamic phase definitions
    let dynamic_phase_definitions: Vec<DynamicPhaseDefinition> = match args
        .dynamic_phases
        .iter()
        .map(|s| DynamicPhaseDefinition::parse(s))
        .collect::<walltime_core::Result<Vec<_>>>()
    {
        Ok(defs) => defs,
        Err(e) => {
            eprintln!("{}: {e}", "error".red().bold());
            return ExitCode::from(2);
        }
    };

    let min_phase_duration = std::time::Duration::from_secs_f64(args.min_phase);

    // Build run config
    let command = args.command[0].clone();
    let cmd_args = args.command[1..].to_vec();

    let stdout_is_tty = std::io::IsTerminal::is_terminal(&std::io::stdout());

    let config = RunConfig {
        command: command.clone(),
        args: cmd_args.clone(),
        timestamps: args.timestamps,
        timestamp_format: args.timestamp_format.clone(),
        from_zero: args.from_zero,
        phase_definitions,
        dynamic_phase_definitions,
        force_color: stdout_is_tty,
        min_phase_duration,
        color_output: stdout_is_tty,
    };

    // Run the command
    let result = match runner::run(config).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{}: {e}", "error".red().bold());
            return ExitCode::from(2);
        }
    };

    let exit_code = result.exit_code.unwrap_or(1);

    // Handle history
    let full_command = args.command.clone();
    let log_path = Path::new(&args.log_file);

    let history = if !args.no_log {
        // Load existing history
        let mut hist = match history::load_history(log_path, &full_command) {
            Ok(h) => h,
            Err(e) => {
                eprintln!("{}: failed to load history: {e}", "warning".yellow().bold());
                Vec::new()
            }
        };

        // Save current run
        let entry = HistoryEntry {
            command: full_command,
            started_at: result.started_at.to_utc(),
            total_duration_ms: result.total.as_millis() as u64,
            phases: result
                .phases
                .iter()
                .map(|p| PhaseTime {
                    name: p.name.clone(),
                    duration_ms: p.duration.as_millis() as u64,
                })
                .collect(),
            exit_code: result.exit_code,
        };

        if let Err(e) = history::append_entry(log_path, &entry) {
            eprintln!("{}: failed to save history: {e}", "warning".yellow().bold());
        }

        hist.push(entry);
        hist
    } else {
        Vec::new()
    };

    // Print summary
    if !args.no_summary {
        let summary_text = summary::format_summary(
            &result,
            &history,
            &args.command,
            min_phase_duration,
            args.show_all,
        );

        if atty_check() {
            eprint!("{}", summary_text.dimmed());
        } else {
            eprint!("{summary_text}");
        }
    }

    ExitCode::from(exit_code as u8)
}

/// Check if stderr is a terminal.
fn atty_check() -> bool {
    std::io::IsTerminal::is_terminal(&std::io::stderr())
}
