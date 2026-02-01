use anyhow::Result;
use std::path::PathBuf;

/// Get the application data directory for storing footnote vaults.
/// Uses the standard documents directory on each platform.
pub fn get_app_dir() -> Result<PathBuf> {
    dirs::document_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine documents directory"))
}
