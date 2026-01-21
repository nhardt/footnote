use std::path::PathBuf;

pub fn get_app_dir() -> anyhow::Result<PathBuf> {
    dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))
}

pub const SHARE_SHEET_SUPPORTED: bool = false;
pub fn share_contact_file(_file_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    Err("Share functionality not available on this platform".into())
}
