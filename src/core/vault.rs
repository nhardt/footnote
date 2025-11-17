use std::path::PathBuf;

const VAULT_DIR: &str = "fieldnotes-vault";

/// Get the base vault directory path
pub fn get_vault_path() -> anyhow::Result<PathBuf> {
    let home = std::env::var("HOME")
        .map_err(|_| anyhow::anyhow!("HOME environment variable not set"))?;
    Ok(PathBuf::from(home).join(VAULT_DIR))
}

/// Get the .keys directory path
pub fn get_keys_dir() -> anyhow::Result<PathBuf> {
    Ok(get_vault_path()?.join(".keys"))
}

/// Get the identity.md file path (for "me")
pub fn get_identity_path() -> anyhow::Result<PathBuf> {
    Ok(get_vault_path()?.join("identity.md"))
}

/// Get the devices directory path (for "me")
pub fn get_devices_dir() -> anyhow::Result<PathBuf> {
    Ok(get_vault_path()?.join("devices"))
}

/// Get the notes directory path (for "me")
pub fn get_notes_dir() -> anyhow::Result<PathBuf> {
    Ok(get_vault_path()?.join("notes"))
}

/// Get the outpost directory path
pub fn get_outpost_dir() -> anyhow::Result<PathBuf> {
    Ok(get_vault_path()?.join("outpost"))
}

/// Get the directory path for a specific user's outpost
pub fn get_user_outpost_dir(user_name: &str) -> anyhow::Result<PathBuf> {
    Ok(get_outpost_dir()?.join(user_name))
}

/// Get the identity.md file path for a specific user's outpost
pub fn get_user_identity_path(user_name: &str) -> anyhow::Result<PathBuf> {
    Ok(get_user_outpost_dir(user_name)?.join("identity.md"))
}

/// Get the devices directory path for a specific user's outpost
pub fn get_user_devices_dir(user_name: &str) -> anyhow::Result<PathBuf> {
    Ok(get_user_outpost_dir(user_name)?.join("devices"))
}

/// Get the notes directory path for a specific user's outpost
pub fn get_user_notes_dir(user_name: &str) -> anyhow::Result<PathBuf> {
    Ok(get_user_outpost_dir(user_name)?.join("notes"))
}
