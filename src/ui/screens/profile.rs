use crate::core::crypto;
use crate::core::user::export_me_json_pretty;
use crate::ui::components::device_add_flow::{DeviceAddFlow, DeviceAddState};
use crate::ui::components::device_list_item::DeviceListItem;
use crate::ui::context::VaultContext;
use dioxus::prelude::*;
use dioxus_clipboard::prelude::use_clipboard;

#[derive(Clone, PartialEq)]
pub enum SyncStatus {
    Idle,
    Syncing { device_name: String },
    Success { device_name: String },
    Error { device_name: String, error: String },
}

#[component]
pub fn ProfileScreen() -> Element {
    let mut self_contact = use_signal(|| None::<crypto::ContactRecord>);
    let device_add_state = use_signal(|| DeviceAddState::Idle);
    let sync_status = use_signal(|| SyncStatus::Idle);
    let confirm_delete = use_signal(|| None::<String>);
    let reload_trigger = use_signal(|| 0);
    let vault_ctx = use_context::<VaultContext>();

    // Load contacts on mount and when reload_trigger changes
    use_effect(move || {
        let _ = reload_trigger(); // Subscribe to changes
        let vault_ctx = vault_ctx.clone();
        spawn(async move {
            let vault_path = match vault_ctx.get_vault() {
                Some(path) => path.join(".footnotes"),
                None => return,
            };

            // Load self contact
            let self_path = vault_path.join("contact.json");
            if let Ok(content) = std::fs::read_to_string(&self_path) {
                if let Ok(contact) =
                    serde_json::from_str::<crate::core::crypto::ContactRecord>(&content)
                {
                    self_contact.set(Some(contact));
                }
            }
        });
    });

    rsx! {
        div { class: "max-w-4xl mx-auto p-6",

            h1 { class: "text-2xl font-bold text-zinc-100 mb-6", "Profile" }

            div { class: "mb-6",
                button {
                    class: "px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-700",
                    onclick: {
                        let vault_ctx = vault_ctx.clone();
                        move |_| {
                            let vault_ctx = vault_ctx.clone();
                            spawn(async move {
                                let vault_path = match vault_ctx.get_vault() {
                                    Some(path) => path,
                                    None => {
                                        tracing::error!("No vault path available");
                                        return;
                                    }
                                };

                                if let Err(e) = copy_contact_record_clipboard(&vault_path).await {
                                    tracing::error!("Failed to copy contact record: {}", e);
                                }
                            });
                        }
                    },
                    "Share Contact"
                }
            }

            div { class: "mb-8",
                DeviceAddFlow {
                    device_add_state,
                    reload_trigger,
                }

                if let Some(ref contact) = *self_contact.read() {
                    div { class: "space-y-2",
                        for device in contact.devices.iter() {
                            {
                                let device_name = device.device_name.clone();
                                let endpoint_id = device.iroh_endpoint_id.clone();
                                let is_current = vault_ctx.get_vault()
                                    .and_then(|vp| crate::core::device::get_local_device_name(&vp).ok())
                                    .map(|name| name == device_name)
                                    .unwrap_or(false);

                                rsx! {
                                    DeviceListItem {
                                        key: "{endpoint_id}",
                                        device_name,
                                        endpoint_id,
                                        is_current,
                                        sync_status,
                                        confirm_delete,
                                    }
                                }
                            }
                        }
                    }
                } else {
                    div { class: "text-zinc-400 italic", "Loading..." }
                }
            }

            // Confirmation dialog for device deletion
            if let Some(device_to_delete) = confirm_delete().clone() {
                div { class: "fixed inset-0 bg-zinc-950 bg-opacity-75 flex items-center justify-center z-50",
                    div { class: "bg-zinc-800 rounded-lg p-6 max-w-md border border-zinc-700",
                        h3 { class: "text-lg font-bold text-zinc-100 mb-4", "Delete Device" }
                        p { class: "text-zinc-200 mb-4", "Are you sure you want to delete device '{device_to_delete}'?" }
                        div { class: "flex gap-2 justify-end",
                            button {
                                class: "px-4 py-2 bg-zinc-700 rounded-md hover:bg-zinc-700",
                                onclick: {
                                    let mut confirm_delete = confirm_delete.clone();
                                    move |_| {
                                        confirm_delete.set(None);
                                    }
                                },
                                "Cancel"
                            }
                            button {
                                class: "px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-600",
                                onclick: {
                                    let device_name = device_to_delete.clone();
                                    let mut confirm_delete = confirm_delete.clone();
                                    let mut reload_trigger = reload_trigger.clone();
                                    let vault_ctx = vault_ctx.clone();
                                    move |_| {
                                        let device_name = device_name.clone();
                                        let vault_ctx = vault_ctx.clone();
                                        spawn(async move {
                                            let vault_path = match vault_ctx.get_vault() {
                                                Some(path) => path,
                                                None => {
                                                    tracing::error!("No vault path available for delete");
                                                    confirm_delete.set(None);
                                                    return;
                                                }
                                            };

                                            match crate::core::device::delete_device(&vault_path, &device_name).await {
                                                Ok(_) => {
                                                    reload_trigger.set(reload_trigger() + 1);
                                                    confirm_delete.set(None);
                                                }
                                                Err(e) => {
                                                    tracing::error!("Failed to delete device: {}", e);
                                                    confirm_delete.set(None);
                                                }
                                            }
                                        });
                                    }
                                },
                                "Delete"
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn copy_contact_record_clipboard(vault_path: &std::path::Path) -> anyhow::Result<()> {
    let json_str = export_me_json_pretty(vault_path).await?;
    let mut clipboard = use_clipboard();
    let _ = clipboard.set(json_str);
    Ok(())
}
