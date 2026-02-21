use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use uuid::Uuid;

use crate::util::lamport_timestamp::LamportTimestamp;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tombstone {
    pub uuid: Uuid,
    pub deleted_at: LamportTimestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TombstoneList {
    pub entries: Vec<Tombstone>,
}

impl TombstoneList {
    pub fn load(vault_path: &Path) -> Result<Self> {
        let tombstone_path = vault_path.join(".footnote").join("tombstones.json");
        if !tombstone_path.exists() {
            return Ok(Self::default());
        }
        let json = fs::read_to_string(tombstone_path)?;
        Ok(serde_json::from_str(&json)?)
    }

    pub fn save(&self, vault_path: &Path) -> Result<()> {
        let tombstone_path = vault_path.join(".footnote").join("tombstones.json");
        if let Some(parent) = tombstone_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        let tmp = tombstone_path.with_extension("json.tmp");
        fs::write(&tmp, json)?;
        fs::rename(tmp, tombstone_path)?;
        Ok(())
    }

    pub fn insert(&mut self, uuid: Uuid, deleted_at: Option<LamportTimestamp>) {
        self.entries.retain(|t| t.uuid != uuid);
        if let Some(deleted_at) = deleted_at {
            self.entries.push(Tombstone { uuid, deleted_at });
        } else {
            self.entries.push(Tombstone {
                uuid,
                deleted_at: LamportTimestamp::now(),
            });
        }
    }

    pub fn remove(&mut self, uuid: &Uuid) {
        self.entries.retain(|t| &t.uuid != uuid);
    }

    pub fn is_deleted(&self, uuid: &Uuid) -> bool {
        self.entries.iter().any(|t| &t.uuid == uuid)
    }
}
