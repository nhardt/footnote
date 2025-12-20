use super::needed::VaultStatus;
use crate::ui::context::VaultContext;
use dioxus::prelude::*;
use std::path::PathBuf;

#[component]
pub fn JoinVaultScreen(
    mut vault_status: Signal<VaultStatus>,
    vault_path: PathBuf,
    device_name: String,
    connect_url: String,
) -> Element {
    let mut device_name_input = use_signal(|| device_name);
    let mut connect_url_input = use_signal(|| connect_url);
    let vault_path_display = vault_path.display().to_string();
    let vault_ctx = use_context::<VaultContext>();

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

    let handle_cancel = move |_| {
        vault_status.set(VaultStatus::VaultNeeded);
    };

    rsx! {
        div { class: "flex items-center justify-center h-full",
            div { class: "max-w-md w-full p-8 bg-zinc-800 rounded-lg shadow-lg",
                h1 { class: "text-2xl font-bold mb-6 text-center", "Join Vault" }

                div { class: "mb-4",
                    label { class: "block text-sm font-medium text-zinc-200 mb-2", "Vault Location" }
                    div { class: "px-3 py-2 border border-zinc-600 rounded-md bg-zinc-700 text-zinc-100 font-mono text-sm break-all",
                        "{vault_path_display}"
                    }
                }

                div { class: "mb-4",
                    label { class: "block text-sm font-medium text-zinc-200 mb-2", "Device Name" }
                    input {
                        r#type: "text",
                        class: "w-full px-3 py-2 border border-zinc-600 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-600",
                        placeholder: "e.g., laptop, desktop, phone",
                        value: "{device_name_input}",
                        oninput: move |evt| device_name_input.set(evt.value()),
                        autofocus: true,
                    }
                }

                div { class: "mb-6",
                    label { class: "block text-sm font-medium text-zinc-200 mb-2", "Connection URL" }
                    input {
                        r#type: "text",
                        class: "w-full px-3 py-2 border border-zinc-600 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-600 font-mono text-sm",
                        placeholder: "iroh://...",
                        value: "{connect_url_input}",
                        oninput: move |evt| connect_url_input.set(evt.value()),
                    }
                }

                div { class: "flex gap-2",
                    button {
                        class: "flex-1 px-4 py-2 bg-zinc-800 text-zinc-200 border border-zinc-600 rounded-md hover:bg-zinc-700",
                        onclick: handle_cancel,
                        "Cancel"
                    }
                    button {
                        class: "flex-1 px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-700 disabled:bg-zinc-700 disabled:cursor-not-allowed",
                        disabled: device_name_input().trim().is_empty() || connect_url_input().trim().is_empty(),
                        onclick: handle_join,
                        "Join Vault"
                    }
                }
            }
        }
    }
}
