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

/// A dynamic phase definition parsed from `--dynamic-phase REGEX`.
///
/// The regex must contain at least one capture group. When the pattern matches
/// an output line, the first capture group is used as the phase name.
#[derive(Debug, Clone)]
pub struct DynamicPhaseDefinition {
    /// Regex pattern with a capture group for the phase name.
    pub pattern: Regex,
}

impl DynamicPhaseDefinition {
    /// Parse a dynamic phase definition from a regex string.
    ///
    /// Validates that the regex has at least one capture group.
    pub fn parse(s: &str) -> Result<Self> {
        let pattern = Regex::new(s)?;
        if pattern.captures_len() < 2 {
            return Err(crate::Error::Parse(format!(
                "dynamic phase regex must have at least one capture group: {s:?}"
            )));
        }
        Ok(Self { pattern })
    }
}

/// Tracks phase transitions based on regex matches against output lines.
#[derive(Debug)]
pub struct PhaseTracker {
    definitions: Vec<PhaseDefinition>,
    dynamic_definitions: Vec<DynamicPhaseDefinition>,
    /// Index into `definitions` of the next expected phase.
    next_phase_index: usize,
    /// The currently active phase (name, start time).
    active: Option<(String, Instant)>,
    /// Completed phase records.
    records: Vec<PhaseRecord>,
}

impl PhaseTracker {
    /// Create a new phase tracker with static and dynamic definitions.
    pub fn new(
        definitions: Vec<PhaseDefinition>,
        dynamic_definitions: Vec<DynamicPhaseDefinition>,
    ) -> Self {
        Self {
            definitions,
            dynamic_definitions,
            next_phase_index: 0,
            active: None,
            records: Vec::new(),
        }
    }

    /// Close the currently active phase, recording it.
    fn close_active(&mut self, now: Instant) {
        if let Some((name, start)) = self.active.take() {
            self.records.push(PhaseRecord {
                name,
                start,
                duration: now.duration_since(start),
            });
        }
    }

    /// Process a line of output, checking for phase transitions.
    ///
    /// Returns the name of the new phase if a transition occurred.
    pub fn process_line(&mut self, line: &str, now: Instant) -> Option<String> {
        // Check static (sequential) phases first
        if self.next_phase_index < self.definitions.len() {
            let def = &self.definitions[self.next_phase_index];
            if def.pattern.is_match(line) {
                self.close_active(now);
                let name = self.definitions[self.next_phase_index].name.clone();
                self.active = Some((name.clone(), now));
                self.next_phase_index += 1;
                return Some(name);
            }
        }

        // Check dynamic phase patterns
        for def in &self.dynamic_definitions {
            if let Some(caps) = def.pattern.captures(line)
                && let Some(m) = caps.get(1)
            {
                let name = m.as_str().to_string();
                self.close_active(now);
                self.active = Some((name.clone(), now));
                return Some(name);
            }
        }

        None
    }

    /// Finish tracking, closing any active phase.
    pub fn finish(&mut self, now: Instant) {
        self.close_active(now);
    }

    /// Get the completed phase records.
    pub fn records(&self) -> &[PhaseRecord] {
        &self.records
    }

    /// Returns true if there are no phase definitions (static or dynamic).
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty() && self.dynamic_definitions.is_empty()
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
        let mut tracker = PhaseTracker::new(defs, vec![]);
        let start = Instant::now();

        // First line matches compile phase
        let t1 = start;
        assert_eq!(
            tracker.process_line("Compiling foo v0.1.0", t1),
            Some("compile".to_string())
        );

        // Non-matching line
        let t2 = start + Duration::from_millis(100);
        assert_eq!(tracker.process_line("some other output", t2), None);

        // Matches link phase, closes compile
        let t3 = start + Duration::from_millis(500);
        assert_eq!(
            tracker.process_line("Linking foo", t3),
            Some("link".to_string())
        );

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
        let mut tracker = PhaseTracker::new(vec![], vec![]);
        assert!(tracker.is_empty());
        assert_eq!(tracker.process_line("anything", Instant::now()), None);
    }

    #[test]
    fn parse_dynamic_phase_definition() {
        let def =
            DynamicPhaseDefinition::parse(r"TASK \[([^\]]+)\]").expect("valid dynamic phase def");
        let caps = def
            .pattern
            .captures("TASK [Install packages]")
            .expect("pattern should match");
        assert_eq!(
            caps.get(1).expect("capture group 1").as_str(),
            "Install packages"
        );
    }

    #[test]
    fn parse_dynamic_phase_no_capture_group() {
        let result = DynamicPhaseDefinition::parse(r"TASK \[[^\]]+\]");
        assert!(result.is_err());
    }

    #[test]
    fn phase_tracker_dynamic() {
        let dynamic = vec![DynamicPhaseDefinition::parse(r"TASK \[([^\]]+)\]").expect("valid")];
        let mut tracker = PhaseTracker::new(vec![], dynamic);
        let start = Instant::now();

        assert!(!tracker.is_empty());

        let t1 = start;
        assert_eq!(
            tracker.process_line("TASK [Install packages]", t1),
            Some("Install packages".to_string())
        );

        let t2 = start + Duration::from_millis(200);
        assert_eq!(tracker.process_line("working...", t2), None);

        let t3 = start + Duration::from_millis(500);
        assert_eq!(
            tracker.process_line("TASK [Configure services]", t3),
            Some("Configure services".to_string())
        );

        let t4 = start + Duration::from_millis(1000);
        tracker.finish(t4);

        let records = tracker.records();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "Install packages");
        assert_eq!(records[0].duration, Duration::from_millis(500));
        assert_eq!(records[1].name, "Configure services");
        assert_eq!(records[1].duration, Duration::from_millis(500));
    }

    #[test]
    fn phase_tracker_static_and_dynamic_combined() {
        let static_defs = vec![PhaseDefinition::parse("compile=Compiling").expect("valid")];
        let dynamic = vec![DynamicPhaseDefinition::parse(r"TASK \[([^\]]+)\]").expect("valid")];
        let mut tracker = PhaseTracker::new(static_defs, dynamic);
        let start = Instant::now();

        // Static phase triggers first
        let t1 = start;
        assert_eq!(
            tracker.process_line("Compiling foo", t1),
            Some("compile".to_string())
        );

        // Dynamic phase triggers after static phases exhausted
        let t2 = start + Duration::from_millis(500);
        assert_eq!(
            tracker.process_line("TASK [Deploy]", t2),
            Some("Deploy".to_string())
        );

        let t3 = start + Duration::from_millis(1000);
        tracker.finish(t3);

        let records = tracker.records();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "compile");
        assert_eq!(records[1].name, "Deploy");
    }
}
