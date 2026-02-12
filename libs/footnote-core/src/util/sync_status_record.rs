use crate::util::lamport_timestamp::LamportTimestamp;
use anyhow::Result;
use iroh::PublicKey;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncType {
    Mirror,
    Share,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum SyncDirection {
    Inbound,
    Outbound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncInProgress {
    pub started_at: LamportTimestamp,
    pub files_total: Option<usize>,
    pub files_transferred: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedSync {
    pub completed_at: LamportTimestamp,
    pub files_transferred: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedSync {
    pub failed_at: LamportTimestamp,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatusRecord {
    pub endpoint_id: String,
    pub sync_type: SyncType,
    pub direction: SyncDirection,

    pub current: Option<SyncInProgress>,
    pub last_success: Option<CompletedSync>,
    pub last_failure: Option<FailedSync>,

    #[serde(skip)]
    vault_path: PathBuf,
}

impl SyncStatusRecord {
    pub fn start(
        vault_path: PathBuf,
        endpoint_id: PublicKey,
        sync_type: SyncType,
        direction: SyncDirection,
    ) -> Result<Self> {
        let timestamp = LamportTimestamp::now();
        let existing = Self::read(
            vault_path.clone(),
            &endpoint_id.to_string(),
            direction.clone(),
        )?;

        let status = Self {
            endpoint_id: endpoint_id.to_string(),
            sync_type,
            direction,
            vault_path,
            current: Some(SyncInProgress {
                started_at: timestamp,
                files_total: None,
                files_transferred: 0,
            }),
            last_success: existing.as_ref().and_then(|e| e.last_success.clone()),
            last_failure: existing.as_ref().and_then(|e| e.last_failure.clone()),
        };

        status.write()?;
        Ok(status)
    }

    pub fn read(
        vault_path: PathBuf,
        endpoint_id: &str,
        direction: SyncDirection,
    ) -> Result<Option<Self>> {
        let status_path = Self::status_path(&vault_path, endpoint_id, &direction);

        if !status_path.exists() {
            return Ok(None);
        }

        let json = fs::read_to_string(status_path)?;
        let mut status: Self = serde_json::from_str(&json)?;
        status.vault_path = vault_path;

        Ok(Some(status))
    }

    pub fn update(&mut self, files_transferred: usize, files_total: Option<usize>) -> Result<()> {
        if let Some(current) = &mut self.current {
            current.files_transferred = files_transferred;
            if files_total.is_some() {
                current.files_total = files_total;
            }
        }
        self.write()
    }

    pub fn record_success(mut self) -> Result<()> {
        if let Some(current) = self.current.take() {
            if current.files_transferred > 0 {
                self.last_success = Some(CompletedSync {
                    completed_at: LamportTimestamp::now(),
                    files_transferred: current.files_transferred,
                });
            }
        }
        self.write()
    }

    pub fn record_failure(mut self, reason: &str) -> Result<()> {
        self.current = None;
        self.last_failure = Some(FailedSync {
            failed_at: LamportTimestamp::now(),
            error: reason.to_string(),
        });
        self.write()
    }

    fn status_path(vault_path: &Path, endpoint_id: &str, direction: &SyncDirection) -> PathBuf {
        let direction_str = match direction {
            SyncDirection::Inbound => "inbound",
            SyncDirection::Outbound => "outbound",
        };

        vault_path
            .join(".footnote/status")
            .join(endpoint_id)
            .join(direction_str)
            .join("status.json")
    }

    pub fn write(&self) -> Result<()> {
        let status_path = Self::status_path(&self.vault_path, &self.endpoint_id, &self.direction);

        if let Some(parent) = status_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;

        let temp_path = status_path.with_extension("json.tmp");
        fs::write(&temp_path, json)?;
        fs::rename(temp_path, status_path)?;

        Ok(())
    }
}

pub fn delete_logs_for_endpoint(vault_path: &Path, endpoint: &str) -> Result<()> {
    tracing::info!("removing logs for {}", endpoint);
    let _ = endpoint.parse::<iroh::PublicKey>()?;
    let device_log_path = vault_path.join(".footnote/status").join(endpoint);
    if let Err(e) = fs::remove_dir_all(device_log_path) {
        tracing::info!("failed to remove logs for {}: {}", endpoint, e);
    } else {
        tracing::info!("successfully removed logs for {}", endpoint);
    };
    Ok(())
}
