use std::path::PathBuf;

const FIELDNOTES_DIR: &str = ".fieldnotes";
const LOCAL_DEVICE_KEY_FILE: &str = "this_device";
const CONTACT_FILE: &str = "contact.json";
const OUTPOSTS_DIR: &str = "outposts";
const EMBASSIES_DIR: &str = "embassies";
const NOTES_DIR: &str = "notes";
const IDENTITY_FILE: &str = "identity.md";

/// Get the base vault directory path by searching upward for .fieldnotes/
pub fn get_vault_path() -> anyhow::Result<PathBuf> {
    let mut current_dir = std::env::current_dir()
        .map_err(|_| anyhow::anyhow!("Failed to get current directory"))?;

    loop {
        let fieldnotes_dir = current_dir.join(FIELDNOTES_DIR);
        if fieldnotes_dir.exists() && fieldnotes_dir.is_dir() {
            return Ok(current_dir);
        }

        if !current_dir.pop() {
            anyhow::bail!(
                "No fieldnote vault found. Run 'fieldnote hq create' to initialize a vault."
            );
        }
    }
}

/// Get the .fieldnotes directory path
pub fn get_fieldnotes_dir() -> anyhow::Result<PathBuf> {
    Ok(get_vault_path()?.join(FIELDNOTES_DIR))
}

/// Get the contact.json file path (for "me")
pub fn get_contact_path() -> anyhow::Result<PathBuf> {
    Ok(get_fieldnotes_dir()?.join(CONTACT_FILE))
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
pub fn get_embassy_contact_path(user_name: &str) -> anyhow::Result<PathBuf> {
    Ok(get_embassies_dir()?.join(format!("{}.json", user_name)))
}

/// Get the contact info file path for a specific embassy (old format, deprecated)
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
    let fieldnotes_dir = get_fieldnotes_dir()?;

    if !fieldnotes_dir.exists() {
        anyhow::bail!(
            "Vault not properly initialized at {}. Run 'fieldnote hq create' first.",
            vault_path.display()
        );
    }

    let local_device_key = fieldnotes_dir.join(LOCAL_DEVICE_KEY_FILE);
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
