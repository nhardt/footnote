use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Lamport timestamp for tracking note modifications with causal ordering.
///
/// Uses max(previous_timestamp + 1, current_unix_time) to ensure:
/// - Edits are always logically "after" what was read (causality)
/// - Timestamps stay roughly aligned with wall clock time when possible
/// - Works correctly even when device clocks are skewed
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct LamportTimestamp(pub i64);

impl fmt::Display for LamportTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl LamportTimestamp {
    /// Create a new timestamp for a modification.
    ///
    /// Pass the previous timestamp from the note being modified, or None for new notes.
    /// Returns max(previous + 1, current_wall_clock)
    pub fn new(previous: Option<LamportTimestamp>) -> Self {
        let current_unix = Utc::now().timestamp();
        match previous {
            Some(LamportTimestamp(prev)) => LamportTimestamp(prev.max(current_unix - 1) + 1),
            None => LamportTimestamp(current_unix),
        }
    }

    pub fn now() -> Self {
        let current_unix = Utc::now().timestamp();
        LamportTimestamp(current_unix)
    }

    pub fn as_i64(self) -> i64 {
        self.0
    }

    pub fn to_datetime(self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.0, 0).unwrap_or_else(|| Utc::now())
    }

    // Or if you want a formatted string directly:
    pub fn to_date_string(self) -> String {
        self.to_datetime().format("%Y-%m-%d %H:%M:%S").to_string()
    }
}
