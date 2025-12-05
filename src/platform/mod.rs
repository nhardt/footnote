use std::path::PathBuf;

/// Platform-specific directory picker
///
/// On desktop: Opens a native directory picker dialog
/// On mobile: Returns the default app documents directory
pub async fn pick_directory() -> Option<PathBuf> {
    #[cfg(feature = "desktop")]
    {
        pick_directory_desktop().await
    }

    #[cfg(feature = "mobile")]
    {
        pick_directory_mobile().await
    }

    #[cfg(feature = "web")]
    {
        None
    }
}

#[cfg(feature = "desktop")]
async fn pick_directory_desktop() -> Option<PathBuf> {
    use rfd::AsyncFileDialog;

    AsyncFileDialog::new()
        .set_title("Select Vault Directory")
        .pick_folder()
        .await
        .map(|handle| handle.path().to_path_buf())
}

#[cfg(feature = "mobile")]
async fn pick_directory_mobile() -> Option<PathBuf> {
    dirs::document_dir()
        .map(|dir| dir.join("footnotes"))
}
