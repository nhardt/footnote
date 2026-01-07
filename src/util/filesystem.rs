use crate::model::{note::Note, vault::Vault, vault::VaultState};
use anyhow::Result;
use gethostname::gethostname;
use std::{env, fs, path::PathBuf};

/// mvp: ensure the user has a footnote.wiki in their home directory
/// mvp+1: allow custom via env var
pub fn ensure_default_vault() -> Result<PathBuf> {
    // this doesn't yet "hang together". it would probably be clean to always
    // create a Vault standalone, then state transition to another state. the
    // current task is just to expose joining a vault from a secondary vault,
    // will revisit this later.

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
    match vault.state_read()? {
        VaultState::Uninitialized => {
            Vault::create_standalone(&default_vault_dir)?;
            ()
        }
        _other => (),
    }

    let home_note_file = default_vault_dir.join("home.md");
    if !home_note_file.exists() {
        vault.note_create(&home_note_file, "Welcome to footnote.wiki")?
    }

    Ok(default_vault_dir)
}
