use crate::model::lamport_timestamp::LamportTimestamp;
use crate::model::note::Note;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub uuid: Uuid,
    pub path: PathBuf,
    pub modified: LamportTimestamp,
}

pub type Manifest = HashMap<Uuid, ManifestEntry>;

/// replicas will get all of our files, including things that have been shared
/// with us.
pub fn create_manifest_full(vault_path: &Path) -> Result<Manifest> {
    let mut manifest = Manifest::new();

    for entry in WalkDir::new(vault_path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if file_name.starts_with('.') {
                continue;
            }
        }

        if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        let relative_path = path
            .strip_prefix(vault_path)
            .context("Failed to get relative path")?
            .to_path_buf();

        let Ok(note) = Note::from_path(entry.path(), false) else {
            continue;
        };

        let entry = ManifestEntry {
            uuid: note.frontmatter.uuid,
            path: relative_path,
            modified: note.frontmatter.modified,
        };

        manifest.insert(note.frontmatter.uuid, entry);
    }

    Ok(manifest)
}

/// walk our own notes (not notes replicated to us) and add them to view for the
/// shared_with user
pub fn create_manifest_for_share(vault_path: &Path, shared_with: &str) -> Result<Manifest> {
    let mut manifest = Manifest::new();

    for entry in WalkDir::new(vault_path)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_str().unwrap_or("");
            !name.starts_with('.') && name != "footnotes"
        })
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().is_file() && e.path().extension().and_then(|s| s.to_str()) == Some("md")
        })
    {
        let path = entry.path();
        let Ok(note) = Note::from_path(entry.path(), false) else {
            continue;
        };
        if !note
            .frontmatter
            .share_with
            .contains(&shared_with.to_string())
        {
            continue;
        }

        let relative_path = path
            .strip_prefix(vault_path)
            .context("Failed to get relative path")?
            .to_path_buf();

        let entry = ManifestEntry {
            uuid: note.frontmatter.uuid,
            path: relative_path,
            modified: note.frontmatter.modified,
        };

        manifest.insert(note.frontmatter.uuid, entry);
    }

    Ok(manifest)
}

/// manifest of all files i might read and write to
pub fn create_manifest_local(vault_path: &Path) -> Result<Manifest> {
    let mut manifest = Manifest::new();

    for entry in WalkDir::new(vault_path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if file_name.starts_with('.') {
                continue;
            }
        }

        if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        let relative_path = path
            .strip_prefix(vault_path)
            .context("Failed to get relative path")?
            .to_path_buf();

        // we will start with files created in footnote....
        // it's tempting to re-write all files in the footnote dir.
        let Ok(note) = Note::from_path(entry.path(), false) else {
            continue;
        };

        let entry = ManifestEntry {
            uuid: note.frontmatter.uuid,
            path: relative_path,
            modified: note.frontmatter.modified,
        };

        manifest.insert(note.frontmatter.uuid, entry);
    }

    Ok(manifest)
}

pub fn diff_manifests(local: &Manifest, remote: &Manifest) -> Vec<ManifestEntry> {
    let mut files_to_sync = Vec::new();

    for (uuid, remote_entry) in remote {
        match local.get(uuid) {
            None => {
                files_to_sync.push(remote_entry.clone());
            }
            Some(local_entry) => {
                if remote_entry.modified > local_entry.modified {
                    files_to_sync.push(remote_entry.clone());
                }
            }
        }
    }

    files_to_sync
}

#[cfg(test)]
mod tests {
    use dioxus::html::base;

    use super::*;

    #[test]
    fn test_diff_manifests_new_file() {
        let local = Manifest::new();
        let mut remote = Manifest::new();

        let uuid = Uuid::new_v4();
        remote.insert(
            uuid,
            ManifestEntry {
                uuid,
                path: PathBuf::from("note.md"),
                modified: LamportTimestamp::new(None),
            },
        );

        let diff = diff_manifests(&local, &remote);
        assert_eq!(diff.len(), 1);
        assert_eq!(diff[0].uuid, uuid);
    }

    #[test]
    fn test_diff_manifests_updated_remote() {
        let mut local = Manifest::new();
        let mut remote = Manifest::new();

        let uuid = Uuid::new_v4();
        let base_timestamp = LamportTimestamp::new(None);

        local.insert(
            uuid,
            ManifestEntry {
                path: PathBuf::from("note.md"),
                uuid,
                modified: base_timestamp.clone(),
            },
        );

        remote.insert(
            uuid,
            ManifestEntry {
                path: PathBuf::from("note.md"),
                uuid,
                modified: LamportTimestamp::new(Some(base_timestamp)),
            },
        );

        let diff = diff_manifests(&local, &remote);
        assert_eq!(diff.len(), 1);
    }

    #[test]
    fn test_diff_manifests_local_newer() {
        let mut local = Manifest::new();
        let mut remote = Manifest::new();

        let uuid = Uuid::new_v4();
        let old_time = LamportTimestamp(1000);
        let new_time = LamportTimestamp(2000);

        local.insert(
            uuid,
            ManifestEntry {
                uuid,
                path: PathBuf::from("note.md"),
                modified: new_time,
            },
        );

        remote.insert(
            uuid,
            ManifestEntry {
                path: PathBuf::from("note.md"),
                uuid,
                modified: old_time,
            },
        );

        let diff = diff_manifests(&local, &remote);
        assert_eq!(diff.len(), 0);
    }

    #[test]
    fn test_diff_manifests_no_changes() {
        let mut local = Manifest::new();
        let mut remote = Manifest::new();

        let uuid = Uuid::new_v4();
        let time = LamportTimestamp(1500);

        local.insert(
            uuid,
            ManifestEntry {
                path: PathBuf::from("note.md"),
                uuid,
                modified: time.clone(),
            },
        );

        remote.insert(
            uuid,
            ManifestEntry {
                path: PathBuf::from("note.md"),
                uuid,
                modified: time,
            },
        );

        let diff = diff_manifests(&local, &remote);
        assert_eq!(diff.len(), 0);
    }
}
