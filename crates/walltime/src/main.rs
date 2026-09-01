//! The `walltime` binary. See the crate documentation for usage.

mod args;

use std::path::Path;
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};

use clap::Parser;
use owo_colors::OwoColorize;

use walltime::history::{self, HistoryEntry, PhaseTime};
use walltime::phase::{DynamicPhaseDefinition, PhaseDefinition};
use walltime::runner::{self, RunConfig};
use walltime::summary;

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
        .collect::<walltime::Result<Vec<_>>>()
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
        .collect::<walltime::Result<Vec<_>>>()
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
