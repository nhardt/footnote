use std::path::PathBuf;

pub fn get_app_dir() -> anyhow::Result<PathBuf> {
    dirs::document_dir().ok_or_else(|| anyhow::anyhow!("Could not determine documents directory"))
}
