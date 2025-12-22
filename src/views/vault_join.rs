use crate::components::DirectoryBrowser;
use crate::context::VaultContext;
use crate::Route;
use dioxus::prelude::*;
use std::path::PathBuf;

#[component]
pub fn VaultJoin() -> Element {
    let mut vault_ctx = use_context::<VaultContext>();
    let nav = navigator();

    rsx! {
        DirectoryBrowser {
            action_label: "Local Directory To Mirror To".to_string(),
            is_valid: move |path: PathBuf| path.join(".footnotes").exists(),
            on_select: move |path| {
                vault_ctx.set_vault(path);
                nav.replace(Route::Editor {});
            },
            on_cancel: move |_| {
                nav.replace(Route::VaultHome {});
            }
        }
    }
}

fn handle_join() {
    let handle_join = move |_| {
        if device_name_input().trim().is_empty() || connect_url_input().trim().is_empty() {
            return;
        }

        let device = device_name_input().trim().to_string();
        let url = connect_url_input().trim().to_string();
        let vault_path = vault_path.clone();
        let mut vault_status = vault_status.clone();
        let mut vault_ctx = vault_ctx.clone();

        spawn(async move {
            if let Err(e) = std::env::set_current_dir(&vault_path) {
                vault_status.set(VaultStatus::Error(format!(
                    "Failed to set working directory: {}",
                    e
                )));
                return;
            }

            match crate::core::device::create_remote(&vault_path, &url, &device).await {
                Ok(_) => {
                    vault_ctx.set_vault(vault_path);
                    vault_status.set(VaultStatus::VaultNeeded);
                }
                Err(e) => {
                    vault_status.set(VaultStatus::Error(format!("Failed to join vault: {}", e)));
                }
            }
        });
    };
}
