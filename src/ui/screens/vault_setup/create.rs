use dioxus::prelude::*;
use std::path::PathBuf;
use super::needed::VaultStatus;
use crate::ui::context::VaultContext;

#[component]
pub fn CreateVaultScreen(mut vault_status: Signal<VaultStatus>, vault_path: PathBuf) -> Element {
    let mut device_name = use_signal(|| String::new());
    let vault_path_display = vault_path.display().to_string();
    let vault_ctx = use_context::<VaultContext>();

    let handle_create = move |_| {
        if device_name().trim().is_empty() {
            return;
        }

        let device = device_name().trim().to_string();
        let vault_path = vault_path.clone();
        let mut vault_status = vault_status.clone();
        let mut vault_ctx = vault_ctx.clone();

        spawn(async move {
            match crate::core::init::init(Some(vault_path.clone()), Some("me"), Some(&device)).await
            {
                Ok(_) => {
                    // Set the vault as working directory
                    if let Err(e) = std::env::set_current_dir(&vault_path) {
                        vault_status.set(VaultStatus::Error(format!(
                            "Failed to set working directory: {}",
                            e
                        )));
                        return;
                    }

                    vault_ctx.set_vault(vault_path);
                    vault_status.set(VaultStatus::VaultNeeded);
                }
                Err(e) => {
                    vault_status.set(VaultStatus::Error(format!(
                        "Failed to initialize vault: {}",
                        e
                    )));
                }
            }
        });
    };

    let handle_cancel = move |_| {
        vault_status.set(VaultStatus::VaultNeeded);
    };

    rsx! {
        div { class: "flex items-center justify-center h-full",
            div { class: "max-w-md w-full p-8 bg-app-panel rounded-lg shadow-lg",
                h1 { class: "text-2xl font-bold mb-6 text-center", "Create New Vault" }

                div { class: "mb-4",
                    label { class: "block text-sm font-medium text-app-text-secondary mb-2", "Vault Location" }
                    div { class: "px-3 py-2 border border-app-border-subtle rounded-md bg-app-hover text-app-text font-mono text-sm break-all",
                        "{vault_path_display}"
                    }
                }

                div { class: "mb-6",
                    label { class: "block text-sm font-medium text-app-text-secondary mb-2", "Device Name" }
                    input {
                        r#type: "text",
                        class: "w-full px-3 py-2 border border-app-border-subtle rounded-md focus:outline-none focus:ring-2 focus:ring-app-primary",
                        placeholder: "e.g., laptop, desktop, phone",
                        value: "{device_name}",
                        oninput: move |evt| device_name.set(evt.value()),
                        autofocus: true,
                    }
                }

                div { class: "flex gap-2",
                    button {
                        class: "flex-1 px-4 py-2 bg-app-panel text-app-text-secondary border border-app-border-subtle rounded-md hover:bg-app-hover",
                        onclick: handle_cancel,
                        "Cancel"
                    }
                    button {
                        class: "flex-1 px-4 py-2 bg-app-primary text-white rounded-md hover:bg-app-primary-hover disabled:bg-app-hover disabled:cursor-not-allowed",
                        disabled: device_name().trim().is_empty(),
                        onclick: handle_create,
                        "Create Vault"
                    }
                }
            }
        }
    }
}
