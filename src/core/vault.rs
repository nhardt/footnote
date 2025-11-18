use std::path::PathBuf;

const VAULT_DIR: &str = "fieldnotes-vault";
const LOCAL_DEVICE_KEY_FILE: &str = "this_device";
const OUTPOSTS_DIR: &str = "outposts";
const EMBASSIES_DIR: &str = "embassies";
const NOTES_DIR: &str = "notes";
const KEYS_DIR: &str = ".keys";
const IDENTITY_FILE: &str = "identity.md";

/// Get the base vault directory path
pub fn get_vault_path() -> anyhow::Result<PathBuf> {
    let home = std::env::var("HOME")
        .map_err(|_| anyhow::anyhow!("HOME environment variable not set"))?;
    Ok(PathBuf::from(home).join(VAULT_DIR))
}

/// Get the .keys directory path
pub fn get_keys_dir() -> anyhow::Result<PathBuf> {
    Ok(get_vault_path()?.join(KEYS_DIR))
}

/// Get the identity.md file path (for "me")
pub fn get_identity_path() -> anyhow::Result<PathBuf> {
    Ok(get_vault_path()?.join(IDENTITY_FILE))
}

/// Get the outposts directory path (my devices)
pub fn get_outposts_dir() -> anyhow::Result<PathBuf> {
    Ok(get_vault_path()?.join(OUTPOSTS_DIR))
}

/// Get the notes directory path (for "me")
pub fn get_notes_dir() -> anyhow::Result<PathBuf> {
    Ok(get_vault_path()?.join(NOTES_DIR))
}

/// Get the embassies directory path (other users)
pub fn get_embassies_dir() -> anyhow::Result<PathBuf> {
    Ok(get_vault_path()?.join(EMBASSIES_DIR))
}

/// Get the directory path for a specific embassy (user)
pub fn get_embassy_dir(user_name: &str) -> anyhow::Result<PathBuf> {
    Ok(get_embassies_dir()?.join(user_name))
}

/// Get the contact info file path for a specific embassy
pub fn get_embassy_info_path(user_name: &str) -> anyhow::Result<PathBuf> {
    Ok(get_embassies_dir()?.join(format!("{}_info.md", user_name)))
}

/// Get the notes directory path for a specific embassy
pub fn get_embassy_notes_dir(user_name: &str) -> anyhow::Result<PathBuf> {
    Ok(get_embassy_dir(user_name)?.join(NOTES_DIR))
}

/// Verify that the vault has been initialized with required structure
pub fn verify_vault_layout() -> anyhow::Result<()> {
    let vault_path = get_vault_path()?;
    if !vault_path.exists() {
        anyhow::bail!(
            "Vault not found at {}. Run 'fieldnote hq create' first.",
            vault_path.display()
        );
    }

    let keys_dir = get_keys_dir()?;
    let local_device_key = keys_dir.join(LOCAL_DEVICE_KEY_FILE);
    if !local_device_key.exists() {
        anyhow::bail!(
            "Device key not found. Run 'fieldnote hq create' first."
        );
    }

    let identity_path = get_identity_path()?;
    if !identity_path.exists() {
        anyhow::bail!(
            "Identity file not found. Run 'fieldnote hq create' first."
        );
    }

    Ok(())
}
