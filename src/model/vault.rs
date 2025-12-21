use std::path::{Path, PathBuf};
use anyhow::Result;

const FOOTNOTES_DIR: &str = ".footnotes";

/// Represents a footnote vault at a specific path
pub struct Vault {
    path: PathBuf,
}

impl Vault {
    /// Check if a path contains a valid vault
    pub fn is_valid(path: &Path) -> bool {
        path.join(FOOTNOTES_DIR).exists()
    }

    /// Open an existing vault at the given path
    pub fn open(path: PathBuf) -> Result<Self> {
        if !Self::is_valid(&path) {
            anyhow::bail!(
                "Not a valid vault: {} (missing .footnotes directory)",
                path.display()
            );
        }

        Ok(Self { path })
    }

    /// Get the vault root path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the .footnotes directory path
    pub fn footnotes_dir(&self) -> PathBuf {
        self.path.join(FOOTNOTES_DIR)
    }
}
