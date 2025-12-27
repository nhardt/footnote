use crate::model::{note::Note, vault::Vault};
use anyhow::Result;
use gethostname::gethostname;
use std::{fs, path::PathBuf};

/// mvp: ensure the user has a footnote.wiki in their home directory
pub fn ensure_default_vault() -> Result<PathBuf> {
    let app_dir = crate::platform::get_app_dir()?;
    let default_vault_dir = app_dir.join("footnote.wiki");

    if !default_vault_dir.exists() {
        fs::create_dir_all(&default_vault_dir)?;
        let hostname = gethostname();
        let _vault = Vault::create_primary(&default_vault_dir, "", &hostname.to_string_lossy())?;
    }

    let home_note_file = default_vault_dir.join("home.md");
    if !home_note_file.exists() {
        let _home_note = Note::create(&home_note_file, "Welcome to footnote.wiki");
    }

    Ok(default_vault_dir)
}
