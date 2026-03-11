//! CLI for measuring time spent in a process.

mod args;

use std::path::Path;
use std::process::ExitCode;

use clap::Parser;
use owo_colors::OwoColorize;

use walltime_core::history::{self, HistoryEntry, PhaseTime};
use walltime_core::phase::PhaseDefinition;
use walltime_core::runner::{self, RunConfig};
use walltime_core::summary;

use args::Args;

#[tokio::main]
async fn main() -> ExitCode {
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

    // Build run config
    let command = args.command[0].clone();
    let cmd_args = args.command[1..].to_vec();

    let force_color = match args.color {
        args::ColorChoice::Always => true,
        args::ColorChoice::Never => false,
        args::ColorChoice::Auto => std::io::IsTerminal::is_terminal(&std::io::stdout()),
    };

    let config = RunConfig {
        command: command.clone(),
        args: cmd_args.clone(),
        timestamps: args.timestamps,
        timestamp_format: args.timestamp_format.clone(),
        from_zero: args.from_zero,
        phase_definitions,
        force_color,
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
        let summary_text = summary::format_summary(&result, &history, &args.command);

        let use_color = match args.color {
            args::ColorChoice::Always => true,
            args::ColorChoice::Never => false,
            args::ColorChoice::Auto => atty_check(),
        };

        if use_color {
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
