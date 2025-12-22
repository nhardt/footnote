use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use walkdir::WalkDir;

use super::note::{self, VectorTime};

/// A single entry in the manifest representing one note file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    /// Relative path from the notes directory
    pub path: PathBuf,

    /// UUID of the document
    pub uuid: Uuid,

    /// Last modification timestamp (vector time)
    pub modified: VectorTime,

    /// BLAKE3 hash of the file content for integrity verification
    pub hash: String,
}

/// A manifest containing all notes in a directory
///
/// Indexed by UUID for efficient lookups and conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Map of UUID -> ManifestEntry
    pub entries: HashMap<Uuid, ManifestEntry>,
}

impl Manifest {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Get the number of entries in the manifest
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the manifest is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Add an entry to the manifest
    pub fn add_entry(&mut self, entry: ManifestEntry) {
        self.entries.insert(entry.uuid, entry);
    }

    /// Get an entry by UUID
    pub fn get(&self, uuid: &Uuid) -> Option<&ManifestEntry> {
        self.entries.get(uuid)
    }
}

/// Reason why a file needs to be synced
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncReason {
    /// File doesn't exist locally
    NewFile,

    /// Remote file is newer than local
    UpdatedRemote,
}

/// A file that needs to be synced
#[derive(Debug, Clone)]
pub struct FileToSync {
    /// Relative path to the file
    pub path: PathBuf,

    /// UUID of the document
    pub uuid: Uuid,

    /// Why this file needs to be synced
    pub reason: SyncReason,
}

/// Create a manifest by scanning a directory for markdown files
///
/// Walks the directory recursively, parses frontmatter from each .md file,
/// and computes BLAKE3 hashes for integrity verification.
pub fn create_manifest(notes_dir: &Path) -> Result<Manifest> {
    let mut manifest = Manifest::new();

    if !notes_dir.exists() {
        return Ok(manifest);
    }

    for entry in WalkDir::new(notes_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Skip hidden directories and files (starting with .)
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if file_name.starts_with('.') {
                continue;
            }
        }

        // Only process markdown files
        if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        // Get relative path from notes directory
        let relative_path = path
            .strip_prefix(notes_dir)
            .context("Failed to get relative path")?
            .to_path_buf();

        // Parse frontmatter
        let frontmatter = note::get_frontmatter(path)
            .with_context(|| format!("Failed to parse frontmatter for {}", path.display()))?;

        // Compute file hash
        let hash =
            hash_file(path).with_context(|| format!("Failed to hash file {}", path.display()))?;

        let entry = ManifestEntry {
            path: relative_path,
            uuid: frontmatter.uuid,
            modified: frontmatter.modified,
            hash,
        };

        manifest.add_entry(entry);
    }

    Ok(manifest)
}

/// Compute BLAKE3 hash of a file
fn hash_file(path: &Path) -> Result<String> {
    let contents = fs::read(path)?;
    let hash = blake3::hash(&contents);
    Ok(hash.to_string())
}

/// Compare two manifests and determine which files need to be synced
///
/// Uses Last-Write-Wins (LWW) conflict resolution based on modification timestamps.
/// Only returns files that need to be added or updated (never deletions, per design).
///
/// Logic:
/// - If UUID exists in remote but not local: NewFile
/// - If UUID exists in both:
///   - If remote modified > local modified: UpdatedRemote
///   - Otherwise: skip (local is up-to-date or newer)
pub fn diff_manifests(local: &Manifest, remote: &Manifest) -> Vec<FileToSync> {
    let mut files_to_sync = Vec::new();

    for (uuid, remote_entry) in &remote.entries {
        match local.get(uuid) {
            None => {
                // File doesn't exist locally - need to sync
                files_to_sync.push(FileToSync {
                    path: remote_entry.path.clone(),
                    uuid: *uuid,
                    reason: SyncReason::NewFile,
                });
            }
            Some(local_entry) => {
                // File exists - check if remote is newer
                if remote_entry.modified > local_entry.modified {
                    files_to_sync.push(FileToSync {
                        path: remote_entry.path.clone(),
                        uuid: *uuid,
                        reason: SyncReason::UpdatedRemote,
                    });
                }
                // If local is newer or same, we don't sync (LWW)
            }
        }
    }

    files_to_sync
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_manifests_new_file() {
        let local = Manifest::new();
        let mut remote = Manifest::new();

        let uuid = Uuid::new_v4();
        remote.add_entry(ManifestEntry {
            path: PathBuf::from("note.md"),
            uuid,
            modified: VectorTime::default(),
            hash: "abc123".to_string(),
        });

        let diff = diff_manifests(&local, &remote);
        assert_eq!(diff.len(), 1);
        assert_eq!(diff[0].reason, SyncReason::NewFile);
        assert_eq!(diff[0].uuid, uuid);
    }

    #[test]
    fn test_diff_manifests_updated_remote() {
        let mut local = Manifest::new();
        let mut remote = Manifest::new();

        let uuid = Uuid::new_v4();
        let old_time = VectorTime(1000);
        let new_time = VectorTime(2000);

        local.add_entry(ManifestEntry {
            path: PathBuf::from("note.md"),
            uuid,
            modified: old_time,
            hash: "abc123".to_string(),
        });

        remote.add_entry(ManifestEntry {
            path: PathBuf::from("note.md"),
            uuid,
            modified: new_time,
            hash: "def456".to_string(),
        });

        let diff = diff_manifests(&local, &remote);
        assert_eq!(diff.len(), 1);
        assert_eq!(diff[0].reason, SyncReason::UpdatedRemote);
    }

    #[test]
    fn test_diff_manifests_local_newer() {
        let mut local = Manifest::new();
        let mut remote = Manifest::new();

        let uuid = Uuid::new_v4();
        let old_time = VectorTime(1000);
        let new_time = VectorTime(2000);

        local.add_entry(ManifestEntry {
            path: PathBuf::from("note.md"),
            uuid,
            modified: new_time,
            hash: "abc123".to_string(),
        });

        remote.add_entry(ManifestEntry {
            path: PathBuf::from("note.md"),
            uuid,
            modified: old_time,
            hash: "def456".to_string(),
        });

        let diff = diff_manifests(&local, &remote);
        assert_eq!(diff.len(), 0); // Local is newer, don't sync
    }

    #[test]
    fn test_diff_manifests_no_changes() {
        let mut local = Manifest::new();
        let mut remote = Manifest::new();

        let uuid = Uuid::new_v4();
        let time = VectorTime(1500);

        local.add_entry(ManifestEntry {
            path: PathBuf::from("note.md"),
            uuid,
            modified: time.clone(),
            hash: "abc123".to_string(),
        });

        remote.add_entry(ManifestEntry {
            path: PathBuf::from("note.md"),
            uuid,
            modified: time,
            hash: "abc123".to_string(),
        });

        let diff = diff_manifests(&local, &remote);
        assert_eq!(diff.len(), 0);
    }
}
