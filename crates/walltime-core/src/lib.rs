//! Core library for measuring time spent in a process.
//!
//! Provides the building blocks used by the `wtime` CLI:
//!
//! - **[`runner`]** — spawns a child process, streams its output with optional
//!   timestamps, and collects a [`summary::RunResult`].
//! - **[`phase`]** — tracks sequential or dynamic phases via regex matching
//!   against output lines.
//! - **[`timestamp`]** — formats wall-clock or elapsed-time prefixes for each
//!   output line.
//! - **[`summary`]** — renders the timing summary and history comparison table.
//! - **[`history`]** — persists run entries to a JSONL log and loads them back
//!   for comparison.
//! - **[`error`]** — the unified [`Error`] enum and [`Result`] alias.

pub mod error;
pub mod history;
pub mod phase;
pub mod runner;
pub mod summary;
pub mod timestamp;

pub use error::{Error, Result};
