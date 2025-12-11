use dioxus::prelude::*;
use crate::ui::context::VaultContext;

#[derive(Clone, PartialEq)]
enum ListenStatus {
    Idle,
    Listening { endpoint_id: String },
    Received { from: String, endpoint_id: String },
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
pub fn SyncScreen() -> Element {
    let mut self_contact = use_signal(|| None::<crate::core::crypto::ContactRecord>);
    let mut trusted_contacts =
        use_signal(|| Vec::<(String, crate::core::crypto::ContactRecord)>::new());
    let sync_status = use_signal(|| SyncStatus::Idle);
    let mut listen_status = use_signal(|| ListenStatus::Idle);
    let mut cancel_token = use_signal(|| None::<tokio_util::sync::CancellationToken>);
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

            // Load trusted contacts
            let contacts_dir = vault_path.join("contacts");
            if let Ok(entries) = std::fs::read_dir(contacts_dir) {
                let mut contacts = Vec::new();
                for entry in entries.flatten() {
                    if let Ok(file_name) = entry.file_name().into_string() {
                        if file_name.ends_with(".json") {
                            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                                if let Ok(contact) = serde_json::from_str::<
                                    crate::core::crypto::ContactRecord,
                                >(&content)
                                {
                                    let petname = file_name.trim_end_matches(".json").to_string();
                                    contacts.push((petname, contact));
                                }
                            }
                        }
                    }
                }
                contacts.sort_by(|a, b| a.0.cmp(&b.0));
                trusted_contacts.set(contacts);
            }
        });
    });

    rsx! {
        div { class: "max-w-4xl mx-auto p-6",
            h1 { class: "text-2xl font-bold mb-6", "Device Sync" }

            // Me section
            div { class: "mb-8",
                h2 { class: "text-xl font-bold mb-4", "My Devices" }
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
                                        class: "bg-white border border-gray-200 rounded-md p-4",
                                        div { class: "flex items-center justify-between",
                                            div { class: "flex-1",
                                                div { class: "font-semibold",
                                                    "{device_name}"
                                                    if is_current {
                                                        span { class: "ml-2 text-xs text-green-600 font-normal", "(this device)" }
                                                    }
                                                }
                                                div { class: "text-sm text-gray-600 mt-1 font-mono text-xs truncate",
                                                    "ID: {endpoint_id}"
                                                }
                                                div { class: "text-sm text-gray-500 mt-1",
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
                                                            class: "px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed",
                                                            disabled: !matches!(sync_status(), SyncStatus::Idle),
                                                            onclick: {
                                                                let device_name = device_name.clone();
                                                                let endpoint_id = endpoint_id.clone();
                                                                let vault_ctx = vault_ctx.clone();
                                                                let sync_status = sync_status.clone();
                                                                move |_| {
                                                                    let device_name = device_name.clone();
                                                                    let endpoint_id = endpoint_id.clone();
                                                                    let vault_ctx = vault_ctx.clone();
                                                                    let mut sync_status = sync_status.clone();

                                                                    spawn(async move {
                                                                    sync_status.set(SyncStatus::Syncing { device_name: device_name.clone() });

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
                                                                                    sync_status.set(SyncStatus::Success { device_name: device_name.clone() });
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
                                                            },
                                                            "Sync"
                                                        }
                                                        button {
                                                            class: "px-4 py-2 ml-2 bg-red-600 text-white rounded-md hover:bg-red-700 disabled:bg-gray-300 disabled:cursor-not-allowed",
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
                    div { class: "text-gray-500 italic", "Loading..." }
                }
            }

            // Confirmation dialog for device deletion
            if let Some(device_to_delete) = confirm_delete().clone() {
                div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
                    div { class: "bg-white rounded-lg p-6 max-w-md",
                        h3 { class: "text-lg font-bold mb-4", "Delete Device" }
                        p { class: "mb-4", "Are you sure you want to delete device '{device_to_delete}'?" }
                        div { class: "flex gap-2 justify-end",
                            button {
                                class: "px-4 py-2 bg-gray-300 rounded-md hover:bg-gray-400",
                                onclick: {
                                    let mut confirm_delete = confirm_delete.clone();
                                    move |_| {
                                        confirm_delete.set(None);
                                    }
                                },
                                "Cancel"
                            }
                            button {
                                class: "px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-700",
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

            // Receive Sync section
            div { class: "mb-8",
                h2 { class: "text-xl font-bold mb-4", "Receive Sync" }
                div { class: "bg-white border border-gray-200 rounded-md p-4",
                    div { class: "flex items-center justify-between",
                        div { class: "flex-1",
                            div { class: "font-semibold", "Accept sync from other devices" }
                            div { class: "text-sm text-gray-600 mt-1",
                                match listen_status() {
                                    ListenStatus::Idle => "Not listening".to_string(),
                                    ListenStatus::Listening { ref endpoint_id } => {
                                        format!("Listening on: {}...", &endpoint_id[..16.min(endpoint_id.len())])
                                    }
                                    ListenStatus::Received { ref from, .. } => {
                                        format!("Recently received sync from: {}", from)
                                    }
                                    ListenStatus::Error(ref e) => format!("Error: {}", e),
                                }
                            }
                        }
                        if matches!(listen_status(), ListenStatus::Listening { .. } | ListenStatus::Received { .. }) {
                            button {
                                class: "px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-700",
                                onclick: move |_| {
                                    // Stop listening
                                    if let Some(token) = cancel_token() {
                                        token.cancel();
                                        cancel_token.set(None);
                                        listen_status.set(ListenStatus::Idle);
                                    }
                                },
                                "Stop Listening"
                            }
                        } else {
                            button {
                                class: "px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 disabled:bg-gray-300",
                                disabled: matches!(listen_status(), ListenStatus::Error(_)),
                                onclick: move |_| {
                                    let mut listen_status = listen_status.clone();
                                    let mut cancel_token = cancel_token.clone();
                                    let vault_ctx = vault_ctx.clone();

                                    spawn(async move {
                                        let vault_path = match vault_ctx.get_vault() {
                                            Some(path) => path,
                                            None => {
                                                listen_status.set(ListenStatus::Error("No vault path available".to_string()));
                                                return;
                                            }
                                        };

                                        match crate::core::mirror::listen_background(&vault_path).await {
                                            Ok((mut rx, token)) => {
                                                cancel_token.set(Some(token));

                                                // Consume events from the channel
                                                while let Some(event) = rx.recv().await {
                                                    match event {
                                                        crate::core::mirror::ListenEvent::Started { endpoint_id } => {
                                                            listen_status.set(ListenStatus::Listening { endpoint_id: endpoint_id.clone() });
                                                        }
                                                        crate::core::mirror::ListenEvent::Received { from } => {
                                                            // Keep the endpoint_id when showing received status
                                                            if let ListenStatus::Listening { endpoint_id } = listen_status() {
                                                                listen_status.set(ListenStatus::Received {
                                                                    from: from.clone(),
                                                                    endpoint_id: endpoint_id.clone()
                                                                });

                                                                // Reset to listening after 3 seconds
                                                                let mut listen_status = listen_status.clone();
                                                                let endpoint_id_copy = endpoint_id.clone();
                                                                spawn(async move {
                                                                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                                                                    if matches!(listen_status(), ListenStatus::Received { .. }) {
                                                                        listen_status.set(ListenStatus::Listening { endpoint_id: endpoint_id_copy });
                                                                    }
                                                                });
                                                            }
                                                        }
                                                        crate::core::mirror::ListenEvent::Stopped => {
                                                            listen_status.set(ListenStatus::Idle);
                                                            cancel_token.set(None);
                                                            break;
                                                        }
                                                        crate::core::mirror::ListenEvent::Error(err) => {
                                                            listen_status.set(ListenStatus::Error(err));
                                                            cancel_token.set(None);
                                                            break;
                                                        }
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                listen_status.set(ListenStatus::Error(e.to_string()));
                                            }
                                        }
                                    });
                                },
                                "Start Listening"
                            }
                        }
                    }

                    // Show recent sync info
                    if let ListenStatus::Received { ref from, .. } = listen_status() {
                        div { class: "mt-2 p-2 bg-green-50 border border-green-200 rounded text-sm text-green-700",
                            "Received sync from {from}"
                        }
                    }

                    if let ListenStatus::Error(ref e) = listen_status() {
                        div { class: "mt-2 p-2 bg-red-50 border border-red-200 rounded text-sm text-red-700",
                            "Error: {e}"
                        }
                    }
                }
            }

            // Trusted contacts section
            div {
                h2 { class: "text-xl font-bold mb-4", "Trusted Contacts" }
                if trusted_contacts().is_empty() {
                    div { class: "text-gray-500 italic", "No trusted contacts yet" }
                } else {
                    div { class: "space-y-4",
                        for (petname, contact) in trusted_contacts().iter() {
                            div {
                                key: "{petname}",
                                class: "bg-white border border-gray-200 rounded-md p-4",
                                div { class: "font-semibold mb-2", "{petname} ({contact.username})" }
                                div { class: "space-y-2 ml-4",
                                    for device in contact.devices.iter() {
                                        {
                                            let device_name = device.device_name.clone();
                                            let endpoint_id = device.iroh_endpoint_id.clone();

                                            rsx! {
                                                div {
                                                    key: "{endpoint_id}",
                                                    class: "flex items-center justify-between border-l-2 border-gray-200 pl-3 py-2",
                                                    div { class: "flex-1",
                                                        div { class: "text-sm font-medium", "{device_name}" }
                                                        div { class: "text-xs text-gray-500 font-mono truncate", "ID: {endpoint_id}" }
                                                    }
                                                    div { class: "text-xs text-gray-400", "—" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
