use dioxus::prelude::*;

use anyhow::Result;
use std::collections::HashMap;

use footnote_core::{
    model::{device::Device, vault::Vault},
    util::sync_status_record::{RecentFile, SyncDirection, SyncStatusRecord},
};

#[derive(Clone, Copy)]
pub struct SyncStatusContext {
    statuses: Signal<HashMap<(String, SyncDirection), SyncStatusRecord>>,
    vault_path: Signal<std::path::PathBuf>,
}

impl SyncStatusContext {
    pub fn new(vault: &Vault) -> Self {
        let vault_path = vault.base_path();
        let mut statuses = HashMap::new();

        if let Ok(entries) = std::fs::read_dir(vault_path.join(".footnote").join("status")) {
            for entry in entries.flatten() {
                let endpoint_id = entry.file_name().to_string_lossy().to_string();

                for direction in [SyncDirection::Inbound, SyncDirection::Outbound] {
                    if let Ok(Some(mut status)) =
                        SyncStatusRecord::read(vault_path.clone(), &endpoint_id, direction.clone())
                    {
                        if status.current.is_some() {
                            status.current = None;
                            let _ = status.write();
                        }
                        statuses.insert((endpoint_id.clone(), direction), status);
                    }
                }
            }
        }

        Self {
            statuses: Signal::new(statuses),
            vault_path: Signal::new(vault_path),
        }
    }

    pub fn get(&self, endpoint_id: &str, direction: SyncDirection) -> Option<SyncStatusRecord> {
        self.statuses
            .read()
            .get(&(endpoint_id.to_string(), direction))
            .cloned()
    }

    pub fn reload(&mut self, endpoint_id: &str, direction: SyncDirection) -> Result<()> {
        let vault_path = self.vault_path.read().clone();

        if let Some(status) = SyncStatusRecord::read(vault_path, endpoint_id, direction.clone())? {
            self.statuses
                .write()
                .insert((endpoint_id.to_string(), direction), status);
        }

        Ok(())
    }

    pub fn reload_from(&mut self, status: SyncStatusRecord) {
        let key = (status.endpoint_id.clone(), status.direction.clone());
        self.statuses.write().insert(key, status);
    }

    pub fn reload_all(&mut self) -> Result<()> {
        let vault_path = self.vault_path.read().clone();
        let mut statuses = HashMap::new();

        if let Ok(entries) = std::fs::read_dir(vault_path.join(".footnote").join("status")) {
            for entry in entries.flatten() {
                let endpoint_id = entry.file_name().to_string_lossy().to_string();

                for direction in [SyncDirection::Inbound, SyncDirection::Outbound] {
                    if let Ok(Some(status)) =
                        SyncStatusRecord::read(vault_path.clone(), &endpoint_id, direction.clone())
                    {
                        statuses.insert((endpoint_id.clone(), direction), status);
                    }
                }
            }
        }

        self.statuses.set(statuses);
        Ok(())
    }

    pub fn recent_files_for_devices(&self, devices: &[Device]) -> Vec<RecentFile> {
        let statuses = self.statuses.read();
        let mut files: Vec<RecentFile> = devices
            .iter()
            .filter_map(|d| statuses.get(&(d.iroh_endpoint_id.clone(), SyncDirection::Inbound)))
            .flat_map(|s| s.recent_files.iter().cloned())
            .collect();
        files.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        files.dedup_by_key(|f| f.uuid);
        files.truncate(30);
        files
    }
}
