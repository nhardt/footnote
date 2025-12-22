/// Vector time for tracking modifications with causality
/// Uses max(file_modified_time + 1, unix_time) for conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct VectorTime(pub i64);

impl VectorTime {
    /// Create a new vector time from a previous time
    /// Returns max(previous_time + 1, current_unix_time)
    pub fn new(previous: Option<VectorTime>) -> Self {
        let current_unix = Utc::now().timestamp();
        match previous {
            Some(VectorTime(prev)) => VectorTime(std::cmp::max(prev + 1, current_unix)),
            None => VectorTime(current_unix),
        }
    }

    /// Get the unix timestamp value
    pub fn as_i64(&self) -> i64 {
        self.0
    }
}

impl Default for VectorTime {
    fn default() -> Self {
        VectorTime::new(None)
    }
}
