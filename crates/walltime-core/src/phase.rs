//! Phase tracking for sequential build/process stages.

use regex::Regex;
use std::time::{Duration, Instant};

use crate::Result;

/// A phase definition parsed from `--phase NAME=REGEX`.
#[derive(Debug, Clone)]
pub struct PhaseDefinition {
    /// Human-readable name for this phase.
    pub name: String,
    /// Regex pattern that triggers this phase.
    pub pattern: Regex,
}

impl PhaseDefinition {
    /// Parse a phase definition from a `NAME=REGEX` string.
    pub fn parse(s: &str) -> Result<Self> {
        let (name, pattern_str) = s.split_once('=').ok_or_else(|| {
            crate::Error::Parse(format!("invalid phase format: {s:?} (expected NAME=REGEX)"))
        })?;
        let pattern = Regex::new(pattern_str)?;
        Ok(Self {
            name: name.to_string(),
            pattern,
        })
    }
}

/// A completed phase with its timing information.
#[derive(Debug, Clone)]
pub struct PhaseRecord {
    /// Name of the phase.
    pub name: String,
    /// When the phase started.
    pub start: Instant,
    /// How long the phase lasted.
    pub duration: Duration,
}

/// Tracks phase transitions based on regex matches against output lines.
#[derive(Debug)]
pub struct PhaseTracker {
    definitions: Vec<PhaseDefinition>,
    /// Index into `definitions` of the next expected phase.
    next_phase_index: usize,
    /// The currently active phase and when it started.
    active: Option<(usize, Instant)>,
    /// Completed phase records.
    records: Vec<PhaseRecord>,
}

impl PhaseTracker {
    /// Create a new phase tracker with the given definitions.
    pub fn new(definitions: Vec<PhaseDefinition>) -> Self {
        Self {
            definitions,
            next_phase_index: 0,
            active: None,
            records: Vec::new(),
        }
    }

    /// Process a line of output, checking for phase transitions.
    ///
    /// Returns the name of the new phase if a transition occurred.
    pub fn process_line(&mut self, line: &str, now: Instant) -> Option<&str> {
        // Only look at the next expected phase (sequential phases)
        if self.next_phase_index >= self.definitions.len() {
            return None;
        }

        let def = &self.definitions[self.next_phase_index];
        if !def.pattern.is_match(line) {
            return None;
        }

        // Close the currently active phase
        if let Some((idx, start)) = self.active.take() {
            self.records.push(PhaseRecord {
                name: self.definitions[idx].name.clone(),
                start,
                duration: now.duration_since(start),
            });
        }

        // Start the new phase
        let phase_idx = self.next_phase_index;
        self.active = Some((phase_idx, now));
        self.next_phase_index += 1;

        Some(&self.definitions[phase_idx].name)
    }

    /// Finish tracking, closing any active phase.
    pub fn finish(&mut self, now: Instant) {
        if let Some((idx, start)) = self.active.take() {
            self.records.push(PhaseRecord {
                name: self.definitions[idx].name.clone(),
                start,
                duration: now.duration_since(start),
            });
        }
    }

    /// Get the completed phase records.
    pub fn records(&self) -> &[PhaseRecord] {
        &self.records
    }

    /// Returns true if there are no phase definitions.
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_phase_definition() {
        let def = PhaseDefinition::parse("compile=Compiling").expect("valid phase def");
        assert_eq!(def.name, "compile");
        assert!(def.pattern.is_match("Compiling foo v0.1.0"));
    }

    #[test]
    fn parse_phase_definition_invalid() {
        let result = PhaseDefinition::parse("no-equals-sign");
        assert!(result.is_err());
    }

    #[test]
    fn phase_tracker_sequential() {
        let defs = vec![
            PhaseDefinition::parse("compile=Compiling").expect("valid"),
            PhaseDefinition::parse("link=Linking").expect("valid"),
        ];
        let mut tracker = PhaseTracker::new(defs);
        let start = Instant::now();

        // First line matches compile phase
        let t1 = start;
        assert_eq!(
            tracker.process_line("Compiling foo v0.1.0", t1),
            Some("compile")
        );

        // Non-matching line
        let t2 = start + Duration::from_millis(100);
        assert_eq!(tracker.process_line("some other output", t2), None);

        // Matches link phase, closes compile
        let t3 = start + Duration::from_millis(500);
        assert_eq!(tracker.process_line("Linking foo", t3), Some("link"));

        // Finish
        let t4 = start + Duration::from_millis(1000);
        tracker.finish(t4);

        let records = tracker.records();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "compile");
        assert_eq!(records[0].duration, Duration::from_millis(500));
        assert_eq!(records[1].name, "link");
        assert_eq!(records[1].duration, Duration::from_millis(500));
    }

    #[test]
    fn phase_tracker_no_definitions() {
        let mut tracker = PhaseTracker::new(vec![]);
        assert!(tracker.is_empty());
        assert_eq!(tracker.process_line("anything", Instant::now()), None);
    }
}
