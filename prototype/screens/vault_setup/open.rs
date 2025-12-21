use super::needed::VaultStatus;
use crate::ui::context::VaultContext;
use dioxus::prelude::*;
use std::path::PathBuf;

#[component]
pub fn OpenVaultScreen(mut vault_status: Signal<VaultStatus>, vault_path: PathBuf) -> Element {
    let vault_path_display = vault_path.display().to_string();
    let vault_ctx = use_context::<VaultContext>();

    // Auto-open vault on mount
    use_effect(move || {
        let vault_path = vault_path.clone();
        let mut vault_status = vault_status.clone();
        let mut vault_ctx = vault_ctx.clone();

        spawn(async move {
            // Validate this is a vault directory
            let footnotes_dir = vault_path.join(".footnotes");
            if !footnotes_dir.exists() {
                vault_status.set(VaultStatus::Error(format!(
                    "Not a valid vault: {} (missing .footnotes directory)",
                    vault_path.display()
                )));
                return;
            }

            // Set the vault as working directory
            if let Err(e) = std::env::set_current_dir(&vault_path) {
                vault_status.set(VaultStatus::Error(format!(
                    "Failed to set working directory: {}",
                    e
                )));
                return;
            }

            // Get the local device name
            let device_name = match crate::core::device::get_local_device_name(&vault_path) {
                Ok(name) => name,
                Err(e) => {
                    vault_status.set(VaultStatus::Error(format!(
                        "Failed to get device name: {}",
                        e
                    )));
                    return;
                }
            };

            // Check for device-specific home file, create if it doesn't exist
            let home_filename = format!("home-{}.md", device_name);
            let home_path = vault_path.join(&home_filename);

            if !home_path.exists() {
                // Create device-specific home file
                let uuid = uuid::Uuid::new_v4();
                let vector_time = crate::core::note::VectorTime::default();
                let home_content = format!(
                    r#"---
uuid: {}
modified: {}
share_with: []
---

# Home ({})

Welcome to footnote! This is your home note.
"#,
                    uuid,
                    vector_time.as_i64(),
                    device_name
                );

                if let Err(e) = std::fs::write(&home_path, home_content) {
                    vault_status.set(VaultStatus::Error(format!(
                        "Failed to create home file: {}",
                        e
                    )));
                    return;
                }
            }

            vault_ctx.set_vault(vault_path.clone());

            // Save vault to config
            spawn(async move {
                let config = crate::ui::config::AppConfig {
                    last_vault_path: vault_path,
                    last_file: None,
                };
                if let Err(e) = config.save() {
                    tracing::warn!("Failed to save config: {}", e);
                }
            });

            vault_status.set(VaultStatus::VaultNeeded);
        });
    });

    rsx! {
        div { class: "flex items-center justify-center h-full",
            div { class: "text-center",
                div { class: "text-lg font-medium text-zinc-200", "Opening vault..." }
                div { class: "text-sm text-zinc-400 mt-2", "{vault_path_display}" }
            }
        }
    }
}
