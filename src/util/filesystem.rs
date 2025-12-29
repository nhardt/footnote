use crate::model::{note::Note, vault::Vault, vault::VaultState};
use anyhow::Result;
use gethostname::gethostname;
use std::{fs, path::PathBuf};

/// mvp: ensure the user has a footnote.wiki in their home directory
pub fn ensure_default_vault() -> Result<PathBuf> {
    // this doesn't yet "hang together". it would probably be clean to always
    // create a Vault standalone, then state transition to another state. the
    // current task is just to expose joining a vault from a secondary vault,
    // will revisit this later.
    let app_dir = crate::platform::get_app_dir()?;
    let default_vault_dir = app_dir.join("footnote.wiki");
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
