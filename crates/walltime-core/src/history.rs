//! Run history storage and comparison.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::Result;

/// Timing data for a single phase within a run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseTime {
    /// Name of the phase.
    pub name: String,
    /// Duration in milliseconds.
    pub duration_ms: u64,
}

/// A single run's timing data, stored as one JSON line.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Full command and arguments.
    pub command: Vec<String>,
    /// When the run started.
    pub started_at: DateTime<Utc>,
    /// Total duration in milliseconds.
    pub total_duration_ms: u64,
    /// Per-phase timing data.
    pub phases: Vec<PhaseTime>,
    /// Exit code of the child process.
    pub exit_code: Option<i32>,
}

/// Append a history entry as a JSON line to the given file.
pub fn append_entry(path: &Path, entry: &HistoryEntry) -> Result<()> {
    let json = serde_json::to_string(entry)?;
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{json}")?;
    Ok(())
}

/// Load all history entries matching the given command from the file.
///
/// Returns an empty vector if the file doesn't exist.
pub fn load_history(path: &Path, command: &[String]) -> Result<Vec<HistoryEntry>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let entry: HistoryEntry = serde_json::from_str(line)?;
        if entry.command == command {
            entries.push(entry);
        }
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn sample_entry(command: &[&str]) -> HistoryEntry {
        HistoryEntry {
            command: command.iter().map(|s| s.to_string()).collect(),
            started_at: "2026-03-09T10:23:00Z".parse().expect("valid datetime"),
            total_duration_ms: 2345,
            phases: vec![
                PhaseTime {
                    name: "compile".to_string(),
                    duration_ms: 1456,
                },
                PhaseTime {
                    name: "link".to_string(),
                    duration_ms: 544,
                },
            ],
            exit_code: Some(0),
        }
    }

    #[test]
    fn round_trip_entry() {
        let tmp = NamedTempFile::new().expect("tempfile");
        let entry = sample_entry(&["cargo", "build"]);
        append_entry(tmp.path(), &entry).expect("append");

        let loaded = load_history(tmp.path(), &["cargo".into(), "build".into()]).expect("load");
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].total_duration_ms, 2345);
        assert_eq!(loaded[0].phases.len(), 2);
    }

    #[test]
    fn filter_by_command() {
        let tmp = NamedTempFile::new().expect("tempfile");
        append_entry(tmp.path(), &sample_entry(&["cargo", "build"])).expect("append");
        append_entry(tmp.path(), &sample_entry(&["cargo", "test"])).expect("append");
        append_entry(tmp.path(), &sample_entry(&["cargo", "build"])).expect("append");

        let loaded = load_history(tmp.path(), &["cargo".into(), "build".into()]).expect("load");
        assert_eq!(loaded.len(), 2);
    }

    #[test]
    fn missing_file_returns_empty() {
        let loaded = load_history(Path::new("/tmp/nonexistent-walltime.jsonl"), &[]).expect("load");
        assert!(loaded.is_empty());
    }
}
