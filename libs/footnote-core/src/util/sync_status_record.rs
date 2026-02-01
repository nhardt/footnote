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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncDirection {
    Inbound,
    Outbound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncState {
    InProgress,
    Spurious,
    Success,
    Failure { error: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatusRecord {
    pub endpoint_id: String,
    pub sync_type: SyncType,
    pub direction: SyncDirection,
    pub timestamp: LamportTimestamp,
    pub files_total: Option<usize>,
    pub files_transferred: usize,
    pub started_at: LamportTimestamp,
    pub state: SyncState,

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

        let record = Self {
            endpoint_id: endpoint_id.to_string(),
            sync_type,
            direction,
            timestamp,
            vault_path,
            files_total: None,
            files_transferred: 0,
            started_at: timestamp,
            state: SyncState::InProgress,
        };

        record.write()?;
        Ok(record)
    }

    pub fn update(&mut self, files_transferred: usize, files_total: Option<usize>) -> Result<()> {
        self.files_transferred = files_transferred;
        if files_total.is_some() {
            self.files_total = files_total;
        }
        self.write()
    }

    pub fn record_success(mut self) -> Result<()> {
        if self.files_transferred > 0 {
            self.state = SyncState::Success;
            self.write()?;
            fs::write(
                self.pointer_dir().join("last_success"),
                self.timestamp.as_i64().to_string(),
            )?;
        } else {
            self.state = SyncState::Spurious;
        }
        fs::write(
            self.pointer_dir().join("last_seen"),
            self.timestamp.as_i64().to_string(),
        )?;
        Ok(())
    }

    pub fn record_failure(mut self, reason: &str) -> Result<()> {
        self.state = SyncState::Failure {
            error: reason.to_string(),
        };
        self.write()?;
        fs::write(
            self.pointer_dir().join("last_failure"),
            self.timestamp.as_i64().to_string(),
        )?;
        fs::write(
            self.pointer_dir().join("last_seen"),
            self.timestamp.as_i64().to_string(),
        )?;
        Ok(())
    }

    fn log_path(&self) -> PathBuf {
        let direction_str = match self.direction {
            SyncDirection::Inbound => "inbound",
            SyncDirection::Outbound => "outbound",
        };

        self.vault_path
            .join(".footnote/status")
            .join(&self.endpoint_id)
            .join(direction_str)
            .join(format!("{}.log", self.timestamp.as_i64()))
    }

    fn pointer_dir(&self) -> PathBuf {
        let direction_str = match self.direction {
            SyncDirection::Inbound => "inbound",
            SyncDirection::Outbound => "outbound",
        };

        self.vault_path
            .join(".footnote/status")
            .join(&self.endpoint_id)
            .join(direction_str)
    }

    fn write(&self) -> Result<()> {
        let log_path = self.log_path();
        if let Some(parent) = log_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        fs::write(log_path, json)?;
        Ok(())
    }

    pub fn last_success(
        vault_path: PathBuf,
        endpoint_id: &str,
        direction: SyncDirection,
    ) -> Result<Option<Self>> {
        Self::read_pointer(&vault_path, endpoint_id, direction, "last_success")
    }

    pub fn last_failure(
        vault_path: PathBuf,
        endpoint_id: &str,
        direction: SyncDirection,
    ) -> Result<Option<Self>> {
        Self::read_pointer(&vault_path, endpoint_id, direction, "last_failure")
    }

    fn read_pointer(
        vault_path: &PathBuf,
        endpoint_id: &str,
        direction: SyncDirection,
        pointer_name: &str,
    ) -> Result<Option<Self>> {
        let direction_str = match direction {
            SyncDirection::Inbound => "inbound",
            SyncDirection::Outbound => "outbound",
        };

        let pointer_path = vault_path
            .join(".footnote/status")
            .join(endpoint_id)
            .join(direction_str)
            .join(pointer_name);

        if !pointer_path.exists() {
            return Ok(None);
        }

        let timestamp_str = fs::read_to_string(pointer_path)?;
        let timestamp: i64 = timestamp_str.trim().parse()?;

        let log_path = vault_path
            .join(".footnote/status")
            .join(endpoint_id)
            .join(direction_str)
            .join(format!("{}.log", timestamp));

        if !log_path.exists() {
            return Ok(None);
        }

        let json = fs::read_to_string(log_path)?;
        let mut record: Self = serde_json::from_str(&json)?;
        record.vault_path = vault_path.clone();

        Ok(Some(record))
    }
}

// TODO: log_rotate()
// TODO: log_rotate(device_id)

pub fn delete_logs_for_endpoint(vault_path: &Path, endpoint: &str) -> Result<()> {
    // TODO: ensure this device does not have active replications.
    // for now, the most basic validation is we'll ensure the path to delete
    // parses as an iroh endpoint
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
