use crate::model::vault::{Vault, VaultState};
use anyhow::Result;
use std::{
    fs,
    path::Path,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

/// mvp: ensure the user has a footnote.wiki in their home directory
/// mvp+1: allow custom via env var
/// mvp+2: if on disk metadata is inconsistent, clean up and start standalone
/// mvp+3: take in path and name (this file is poorly named)
pub fn ensure_vault_at_path(vault_path: &Path, vault_name: &str) -> Result<PathBuf> {
    let default_vault_dir = vault_path.join(vault_name);
    fs::create_dir_all(&default_vault_dir)?;
    let vault = Vault::new(&default_vault_dir)?;

    // taking a pretty loose upgrade path here. if past metadata is incompatible
    // with current reset and start again. recreating device metadata isn't too bad
    // but at some point this will not be viable.
    let mut metadata_needs_reset = false;
    if let Err(e) = vault.device_read() {
        tracing::warn!(
            "reading device would cause start up crash, resetting metadata: {}",
            e
        );
        metadata_needs_reset = true;
    };
    if let Err(e) = vault.contact_read() {
        tracing::warn!(
            "reading device would cause start up crash, resetting metadata: {}",
            e
        );
        metadata_needs_reset = true;
    };
    if metadata_needs_reset {
        let backup_name = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(d) => format!(".footnote.backup.{}", d.as_secs()),
            Err(_) => ".footnote.backup.unknown".to_string(),
        };

        fs::rename(
            vault.base_path().join(".footnote"),
            vault.base_path().join(backup_name),
        )?;
    }

    match vault.state_read()? {
        VaultState::Uninitialized => {
            Vault::create_standalone(&default_vault_dir)?;
        }
        _other => {}
    }

    Ok(default_vault_dir)
}
