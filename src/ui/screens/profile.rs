use crate::ui::context::VaultContext;
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
enum DeviceAddState {
    Idle,
    Listening { join_url: String },
    Connecting,
    ReceivedRequest { device_name: String },
    Verifying,
    Success { device_name: String },
    Error(String),
}

#[derive(Clone, PartialEq)]
enum SyncStatus {
    Idle,
    Syncing { device_name: String },
    Success { device_name: String },
    Error { device_name: String, error: String },
}

#[component]
pub fn ProfileScreen() -> Element {
    let mut self_contact = use_signal(|| None::<crate::core::crypto::ContactRecord>);
    let mut device_add_state = use_signal(|| DeviceAddState::Idle);
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

            // Me section
            div { class: "mb-8",
                div { class: "flex items-center justify-between mb-4",
                    h2 { class: "text-xl font-bold text-zinc-100", "My Devices" }
                    if matches!(device_add_state(), DeviceAddState::Idle) {
                        button {
                            class: "px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-700",
                            onclick: move |_| {
                                let mut device_add_state = device_add_state.clone();
                                let mut reload_trigger = reload_trigger.clone();
                                let vault_ctx = vault_ctx.clone();

                                spawn(async move {
                                    // Get vault path from context
                                    let vault_path = match vault_ctx.get_vault() {
                                        Some(path) => path,
                                        None => {
                                            device_add_state.set(DeviceAddState::Error(
                                                "No vault path available".to_string()
                                            ));
                                            return;
                                        }
                                    };

                                    match crate::core::device::create_primary(&vault_path).await {
                                        Ok(mut rx) => {
                                            // Consume events from the channel
                                            while let Some(event) = rx.recv().await {
                                                match event {
                                                    crate::core::device::DeviceAuthEvent::Listening { join_url } => {
                                                        device_add_state.set(DeviceAddState::Listening { join_url });
                                                    }
                                                    crate::core::device::DeviceAuthEvent::Connecting => {
                                                        device_add_state.set(DeviceAddState::Connecting);
                                                    }
                                                    crate::core::device::DeviceAuthEvent::ReceivedRequest { device_name } => {
                                                        device_add_state.set(DeviceAddState::ReceivedRequest { device_name });
                                                    }
                                                    crate::core::device::DeviceAuthEvent::Verifying => {
                                                        device_add_state.set(DeviceAddState::Verifying);
                                                    }
                                                    crate::core::device::DeviceAuthEvent::Success { device_name } => {
                                                        device_add_state.set(DeviceAddState::Success { device_name });
                                                        // Reload contacts
                                                        reload_trigger.set(reload_trigger() + 1);
                                                        break;
                                                    }
                                                    crate::core::device::DeviceAuthEvent::Error(err) => {
                                                        device_add_state.set(DeviceAddState::Error(err));
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            device_add_state.set(DeviceAddState::Error(e.to_string()));
                                        }
                                    }
                                });
                            },
                            "Add Device"
                        }
                    }
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
                        }
                    }
                } else {
                    div { class: "text-zinc-400 italic", "Loading..." }
                }

                // Device pairing UI
                match device_add_state() {
                    DeviceAddState::Listening { ref join_url } => rsx! {
                        div { class: "mt-4 p-4 bg-zinc-800 border border-yellow-600 rounded-md",
                            div { class: "font-semibold text-zinc-100 mb-2", "🔐 Waiting for device..." }
                            div { class: "text-sm text-zinc-300 mb-2", "Copy this URL to your new device:" }
                            div { class: "font-mono text-xs bg-zinc-900 text-zinc-300 p-2 rounded border border-zinc-700 break-all",
                                "{join_url}"
                            }
                            div { class: "text-sm text-zinc-400 mt-2 italic",
                                "Listening for connection..."
                            }
                        }
                    },
                    DeviceAddState::Connecting => rsx! {
                        div { class: "mt-4 p-4 bg-zinc-800 border border-blue-600 rounded-md",
                            div { class: "font-semibold text-zinc-100", "✓ Device connecting..." }
                        }
                    },
                    DeviceAddState::ReceivedRequest { ref device_name } => rsx! {
                        div { class: "mt-4 p-4 bg-zinc-800 border border-blue-600 rounded-md",
                            div { class: "font-semibold text-zinc-100", "✓ Received request from: {device_name}" }
                        }
                    },
                    DeviceAddState::Verifying => rsx! {
                        div { class: "mt-4 p-4 bg-zinc-800 border border-blue-600 rounded-md",
                            div { class: "font-semibold text-zinc-100", "✓ Verifying..." }
                        }
                    },
                    DeviceAddState::Success { ref device_name } => rsx! {
                        div { class: "mt-4 p-4 bg-zinc-800 border border-green-600 rounded-md",
                            div { class: "font-semibold text-zinc-100", "✓ Device '{device_name}' added successfully!" }
                            button {
                                class: "mt-2 text-sm text-indigo-400 hover:underline",
                                onclick: move |_| device_add_state.set(DeviceAddState::Idle),
                                "Done"
                            }
                        }
                    },
                    DeviceAddState::Error(ref error) => rsx! {
                        div { class: "mt-4 p-4 bg-zinc-800 border border-red-600 rounded-md",
                            div { class: "font-semibold text-red-400", "✗ Error" }
                            div { class: "text-sm text-zinc-300 mt-1", "{error}" }
                            button {
                                class: "mt-2 text-sm text-indigo-400 hover:underline",
                                onclick: move |_| device_add_state.set(DeviceAddState::Idle),
                                "Try Again"
                            }
                        }
                    },
                    DeviceAddState::Idle => rsx! {},
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
