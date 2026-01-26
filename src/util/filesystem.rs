use crate::model::{
    contact::Contact,
    note::Note,
    vault::{Vault, VaultState},
};
use anyhow::Result;
use gethostname::gethostname;
use std::{
    env, fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

/// mvp: ensure the user has a footnote.wiki in their home directory
/// mvp+1: allow custom via env var
/// mvp+2: if on disk metadata is inconsistent, clean up and start standalone
pub fn hack_ensure_default_vault() -> Result<PathBuf> {
    let path_key = "FOOTNOTE_PATH";
    let vault_name_key = "FOOTNOTE_VAULT";

    let vault_path = match env::var(path_key) {
        Ok(ref val) if val.is_empty() => crate::platform::get_app_dir()?,
        Ok(val) => PathBuf::from(val),
        Err(_) => crate::platform::get_app_dir()?,
    };

    let vault_name = match env::var(vault_name_key) {
        Ok(ref val) if val.is_empty() => "footnote.wiki".to_string(),
        Ok(val) => val,
        Err(_) => "footnote.wiki".to_string(),
    };

    let default_vault_dir = vault_path.join(vault_name);
    fs::create_dir_all(&default_vault_dir)?;
    let vault = Vault::new(&default_vault_dir)?;

    // taking a pretty loose upgrade path here. if past metadata is incompatible
    // with current reset and start again. recreating device metadata isn't too bad
    // but at some point this will not be viable.
    let mut metadata_needs_reset = false;
    if let Err(e) = vault.device_read() {
        metadata_needs_reset = true;
    };
    if let Err(e) = vault.contact_read() {
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

    let home_note_file = default_vault_dir.join("home.md");
    if !home_note_file.exists() {
        vault.note_create(&home_note_file, "Welcome to footnote.wiki")?
    }

    Ok(default_vault_dir)
}
