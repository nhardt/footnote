use crate::ui::context::VaultContext;
use crate::ui::screens::profile::SyncStatus;
use dioxus::prelude::*;

#[component]
pub fn DeviceListItem(
    device_name: String,
    endpoint_id: String,
    is_current: bool,
    sync_status: Signal<SyncStatus>,
    confirm_delete: Signal<Option<String>>,
) -> Element {
    let vault_ctx = use_context::<VaultContext>();

    rsx! {
        div {
            key: "{endpoint_id}",
            class: "bg-zinc-800 border border-zinc-700 rounded-md p-4",
            div { class: "flex items-center justify-between",
                div { class: "flex-1",
                    div { class: "font-semibold",
                        "{device_name}"
                        if is_current {
                            span { class: "ml-2 text-xs text-green-600 font-normal", "(this device)" }
                        }
                    }
                    div { class: "text-sm text-zinc-300 mt-1 font-mono text-xs truncate",
                        "ID: {endpoint_id}"
                    }
                    div { class: "text-sm text-zinc-400 mt-1",
                        match sync_status() {
                            SyncStatus::Idle => "Ready to sync".to_string(),
                            SyncStatus::Syncing { device_name: ref syncing_device } if syncing_device == &device_name => "Syncing...".to_string(),
                            SyncStatus::Success { device_name: ref success_device } if success_device == &device_name => "Last sync: just now".to_string(),
                            SyncStatus::Error { device_name: ref error_device, .. } if error_device == &device_name => "Last sync: failed".to_string(),
                            _ => "—".to_string(),
                        }
                    }
                }
                if !is_current {
                    {
                        let device_name = device_name.clone();
                        let endpoint_id = endpoint_id.clone();
                        let vault_ctx = vault_ctx.clone();
                        let sync_status = sync_status.clone();
                        let confirm_delete = confirm_delete.clone();

                        rsx! {
                            button {
                                class: "px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-700 disabled:bg-zinc-700 disabled:cursor-not-allowed",
                                disabled: !matches!(sync_status(), SyncStatus::Idle),
                                onclick: {
                                    let device_name = device_name.clone();
                                    let endpoint_id = endpoint_id.clone();
                                    let vault_ctx = vault_ctx.clone();
                                    let sync_status = sync_status.clone();
                                    move |_| {
                                        sync_with_device(
                                            vault_ctx.clone(),
                                            device_name.clone(),
                                            endpoint_id.clone(),
                                            sync_status.clone(),
                                        );
                                    }
                                },
                                "Sync"
                            }
                            button {
                                class: "px-4 py-2 ml-2 bg-red-600 text-white rounded-md hover:bg-red-600 disabled:bg-zinc-700 disabled:cursor-not-allowed",
                                disabled: confirm_delete().is_some(),
                                onclick: {
                                    let device_name = device_name.clone();
                                    let mut confirm_delete = confirm_delete.clone();
                                    move |_| {
                                        confirm_delete.set(Some(device_name.clone()));
                                    }
                                },
                                "Delete"
                            }
                        }
                    }
                }
            }
            if let SyncStatus::Error { device_name: ref error_device, ref error } = sync_status() {
                if error_device == &device_name {
                    div { class: "mt-2 p-2 bg-red-50 border border-red-200 rounded text-sm text-red-700",
                        "Error: {error}"
                    }
                }
            }
        }
    }
}

fn sync_with_device(
    vault_ctx: VaultContext,
    device_name: String,
    endpoint_id: String,
    mut sync_status: Signal<SyncStatus>,
) {
    spawn(async move {
        sync_status.set(SyncStatus::Syncing {
            device_name: device_name.clone(),
        });

        let vault_path = match vault_ctx.get_vault() {
            Some(path) => path,
            None => {
                sync_status.set(SyncStatus::Error {
                    device_name: device_name.clone(),
                    error: "No vault path".to_string(),
                });
                return;
            }
        };

        let notes_dir = vault_path.clone();
        let footnotes_dir = vault_path.join(".footnotes");
        let key_file = footnotes_dir.join("this_device");

        // Load local secret key
        let secret_key = match std::fs::read(&key_file) {
            Ok(key_bytes) => {
                let key_array: Result<[u8; 32], _> = key_bytes.try_into();
                match key_array {
                    Ok(arr) => iroh::SecretKey::from_bytes(&arr),
                    Err(_) => {
                        sync_status.set(SyncStatus::Error {
                            device_name: device_name.clone(),
                            error: "Invalid key length".to_string(),
                        });
                        return;
                    }
                }
            }
            Err(e) => {
                sync_status.set(SyncStatus::Error {
                    device_name: device_name.clone(),
                    error: format!("Failed to read secret key: {}", e),
                });
                return;
            }
        };

        // Parse endpoint ID
        match endpoint_id.parse::<iroh::PublicKey>() {
            Ok(public_key) => {
                match crate::core::sync::push_to_device(&notes_dir, public_key, secret_key).await {
                    Ok(_) => {
                        sync_status.set(SyncStatus::Success {
                            device_name: device_name.clone(),
                        });
                    }
                    Err(e) => {
                        sync_status.set(SyncStatus::Error {
                            device_name: device_name.clone(),
                            error: e.to_string(),
                        });
                    }
                }
            }
            Err(e) => {
                sync_status.set(SyncStatus::Error {
                    device_name: device_name.clone(),
                    error: format!("Invalid endpoint ID: {}", e),
                });
            }
        }
    });
}
