use std::path::PathBuf;

const FOOTNOTES_DIR: &str = ".footnotes";
const LOCAL_DEVICE_KEY_FILE: &str = "this_device";
const CONTACT_FILE: &str = "contact.json";
const CONTACTS_DIR: &str = "contacts";
const TRUSTED_SOURCES_DIR: &str = "footnotes";

/// Get the base vault directory path by searching upward for .footnotes/
pub fn get_vault_path() -> anyhow::Result<PathBuf> {
    let mut current_dir = std::env::current_dir()
        .map_err(|_| anyhow::anyhow!("Failed to get current directory"))?;

    loop {
        let footnotes_dir = current_dir.join(FOOTNOTES_DIR);
        if footnotes_dir.exists() && footnotes_dir.is_dir() {
            return Ok(current_dir);
        }

        if !current_dir.pop() {
            anyhow::bail!(
                "No footnote vault found. Run 'footnote init' to initialize a vault."
            );
        }
    }
}

/// Get the .footnotes directory path
pub fn get_footnotes_dir() -> anyhow::Result<PathBuf> {
    Ok(get_vault_path()?.join(FOOTNOTES_DIR))
}

/// Get the contact.json file path (for "me")
pub fn get_contact_path() -> anyhow::Result<PathBuf> {
    Ok(get_footnotes_dir()?.join(CONTACT_FILE))
}

/// Get the notes directory path (for "me")
/// Notes are stored at the vault root for Obsidian compatibility
pub fn get_notes_dir() -> anyhow::Result<PathBuf> {
    get_vault_path()
}

/// Get the contacts directory path (inside .footnotes)
pub fn get_contacts_dir() -> anyhow::Result<PathBuf> {
    Ok(get_footnotes_dir()?.join(CONTACTS_DIR))
}

/// Get the contact info file path for a specific trusted user
pub fn get_contact_file_path(petname: &str) -> anyhow::Result<PathBuf> {
    Ok(get_contacts_dir()?.join(format!("{}.json", petname)))
}

/// Get the footnotes directory path (trusted users' shared notes)
pub fn get_trusted_sources_dir() -> anyhow::Result<PathBuf> {
    Ok(get_vault_path()?.join(TRUSTED_SOURCES_DIR))
}

/// Get the directory path for a specific trusted user
pub fn get_trusted_user_dir(petname: &str) -> anyhow::Result<PathBuf> {
    Ok(get_trusted_sources_dir()?.join(petname))
}

/// Verify that the vault has been initialized with required structure
pub fn verify_vault_layout() -> anyhow::Result<()> {
    let vault_path = get_vault_path()?;
    let footnotes_dir = get_footnotes_dir()?;

    if !footnotes_dir.exists() {
        anyhow::bail!(
            "Vault not properly initialized at {}. Run 'footnote init' first.",
            vault_path.display()
        );
    }

    let local_device_key = footnotes_dir.join(LOCAL_DEVICE_KEY_FILE);
    if !local_device_key.exists() {
        anyhow::bail!(
            "Device key not found. Run 'footnote init' first."
        );
    }

    let contact_path = get_contact_path()?;
    if !contact_path.exists() {
        anyhow::bail!(
            "Contact file not found. Run 'footnote init' first."
        );
    }

    Ok(())
}
