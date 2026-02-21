use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::OnceLock;
use uuid::Uuid;

use crate::util::lamport_timestamp::LamportTimestamp;

// todo: architect tombstones in a bit cleaner
static TOMBSTONE_WRITE_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tombstone {
    pub uuid: Uuid,
    pub deleted_at: LamportTimestamp,
}

pub fn tombstones_read(vault_path: &Path) -> Result<Vec<Tombstone>> {
    let path = vault_path.join(".footnote").join("tombstones.json");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let json = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&json)?)
}

pub async fn tombstone_create(
    vault_path: &Path,
    uuid: Uuid,
    deleted_at: LamportTimestamp,
) -> Result<()> {
    let lock = TOMBSTONE_WRITE_LOCK.get_or_init(|| tokio::sync::Mutex::new(()));
    let _guard = lock.lock().await;
    let mut entries = tombstones_read(vault_path)?;
    entries.retain(|t| t.uuid != uuid);
    entries.push(Tombstone { uuid, deleted_at });
    save(vault_path, &entries)
}

pub async fn tombstone_delete(vault_path: &Path, uuid: &Uuid) -> Result<()> {
    let lock = TOMBSTONE_WRITE_LOCK.get_or_init(|| tokio::sync::Mutex::new(()));
    let _guard = lock.lock().await;
    let mut entries = tombstones_read(vault_path)?;
    entries.retain(|t| &t.uuid != uuid);
    save(vault_path, &entries)
}

fn save(vault_path: &Path, entries: &[Tombstone]) -> Result<()> {
    let path = vault_path.join(".footnote").join("tombstones.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(entries)?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json)?;
    fs::rename(tmp, path)?;
    Ok(())
}

//pub fn is_deleted(vault_path: &Path, uuid: &Uuid) -> Result<bool> {
//    Ok(load(vault_path)?.iter().any(|t| &t.uuid == uuid))
//}
