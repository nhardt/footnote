use dioxus::prelude::*;

use footnote_core::model::device::Device;
use footnote_core::model::vault::VaultState;
use footnote_core::util::sync_status_record::SyncDirection;

use crate::context::AppContext;
use crate::modal::confirm_modal::ConfirmModal;
use crate::sync_status_context::SyncStatusContext;

#[component]
pub fn Profile() -> Element {
    let app_context = use_context::<AppContext>();
    let vault_state = app_context.vault_state;

    let mut menu_visible = use_signal(|| false);
    let mut show_edit_username_modal = use_signal(|| false);

    rsx! {
        main {
            class: "flex-1 overflow-y-auto",
            div {
                class: "max-w-3xl mx-auto px-4 py-6 sm:px-6",

                div { class: "mb-8",
                    h1 { class: "text-2xl font-bold font-mono mb-2",
                        "Vault: "
                        span { class: "text-zinc-400", "{vault_state}" }
                    }
                    p { class: "text-sm text-zinc-500",
                        "Manage Vault Local Settings"
                    }
                }

                match *vault_state.read() {
                    VaultState::Uninitialized => rsx! {
                        p { "Vault not initialized" }
                    },

                    VaultState::StandAlone => rsx! {
                        div { class: "border border-zinc-800 rounded-lg bg-zinc-900/30 p-8",
                            p { class: "text-zinc-300 mb-8",
                                "You are using Footnote in standalone mode. Select Create Device Group or Join Device Group from the side menu to mirror and share notes."
                            }
                        }
                    },

                    VaultState::SecondaryJoined => rsx! {
                        UserComponent { read_only: true }
                        DeviceListComponent { read_only: true }
                    },

                    VaultState::Primary => rsx! {
                        button {
                            onclick: move |_| {
                                show_edit_username_modal.set(true);
                                menu_visible.set(false);
                            },
                            "Edit Username"
                        }
                        UserComponent { read_only: false }
                        DeviceListComponent { read_only: false }
                    }
                }
            }
        }

        if show_edit_username_modal() {
            EditUsernameModal {
                oncancel: move |_| show_edit_username_modal.set(false),
            }
        }
    }
}

#[component]
fn EditUsernameModal(oncancel: EventHandler) -> Element {
    let mut app_context = use_context::<AppContext>();
    let mut username = use_signal(move || match app_context.vault.read().user_read() {
        Ok(Some(user)) => user.username,
        _ => String::new(),
    });
    let mut err_message = use_signal(|| String::new());

    let save_username = move |_| {
        let vault = app_context.vault.read().clone();
        match vault.user_update(username.read().as_str()) {
            Ok(_) => {
                if let Err(e) = app_context.reload() {
                    tracing::warn!("failed to reload app: {}", e);
                }
                oncancel.call(());
            }
            Err(e) => {
                err_message.set(format!("Error updating username: {}", e));
            }
        }
    };

    rsx! {
        div {
            class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50",
            onclick: move |_| oncancel.call(()),

            div {
                class: "w-full max-w-md border border-zinc-700 rounded-lg bg-zinc-900 shadow-2xl p-6",
                onclick: move |evt| evt.stop_propagation(),

                h3 { class: "text-lg font-semibold mb-4", "Edit Username" }

                input {
                    class: "w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-md text-sm font-mono focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500 mb-4",
                    r#type: "text",
                    placeholder: "Username",
                    value: "{username}",
                    oninput: move |e| username.set(e.value())
                }

                if !err_message().is_empty() {
                    div { class: "mb-4 text-sm text-red-400", "{err_message}" }
                }

                div { class: "flex gap-2 justify-end",
                    button {
                        class: "px-4 py-2 text-sm text-zinc-400 hover:text-zinc-100",
                        onclick: move |_| oncancel.call(()),
                        "Cancel"
                    }
                    button {
                        class: "px-4 py-2 bg-zinc-100 text-zinc-900 rounded-md text-sm font-medium hover:bg-white",
                        onclick: save_username,
                        "Save"
                    }
                }
            }
        }
    }
}

#[component]
fn UserComponent(read_only: bool) -> Element {
    let app_context = use_context::<AppContext>();
    let username = app_context
        .user
        .read()
        .as_ref()
        .map(|u| u.username.clone())
        .unwrap_or_default();

    rsx! {
        section { class: "border border-zinc-800 rounded-lg bg-zinc-900/30 p-6 mb-4",
            div { class: "flex items-center gap-4",
                label { class: "text-sm font-medium text-zinc-300", "Username" }
                span { class: "text-sm font-mono text-zinc-100", "{username}" }
            }
        }
    }
}

#[component]
fn DeviceListComponent(read_only: bool) -> Element {
    let app_context = use_context::<AppContext>();
    let devices = app_context.devices.clone();

    rsx! {
        section { class: "border border-zinc-800 rounded-lg bg-zinc-900/30 overflow-hidden",
            div { class: "p-6 border-b border-zinc-800",
                h2 { class: "text-lg font-semibold font-mono", "My Devices" }
            }
            div { class: "divide-y divide-zinc-800",
                for device in devices.iter() {
                    div { class: "p-6 hover:bg-zinc-900/50 transition-colors group",
                        DeviceRow { device: device.clone(), read_only: read_only }
                    }
                }
            }
        }
    }
}

fn truncate_endpoint_id(id: &str) -> String {
    if id.len() > 9 {
        format!("{}...{}", &id[..4], &id[id.len() - 5..])
    } else {
        id.to_string()
    }
}

#[component]
fn DeviceRow(device: Device, read_only: bool) -> Element {
    let mut app_context = use_context::<AppContext>();
    let (this_device_id, _this_device_name) = app_context.vault.read().device_public_key()?;

    let mut show_edit_modal = use_signal(|| false);

    let sync_status_context = use_context::<SyncStatusContext>();
    let outbound_status =
        sync_status_context.get(&device.iroh_endpoint_id, SyncDirection::Outbound);
    let inbound_status = sync_status_context.get(&device.iroh_endpoint_id, SyncDirection::Inbound);
    let truncated_id = truncate_endpoint_id(&device.iroh_endpoint_id);

    let mut delete_dialog_open = use_signal(|| false);
    let mut delete_dialog_error = use_signal(|| String::new());
    let device_id = device.iroh_endpoint_id.clone();
    let delete_app_context = app_context.clone();
    let delete_device_confirm =
        move || match delete_app_context.vault.read().device_delete(&device_id) {
            Ok(_) => {
                if let Err(e) = app_context.reload() {
                    tracing::warn!("failed to reload app: {}", e);
                }
                delete_dialog_open.set(false);
            }
            Err(e) => {
                delete_dialog_error.set(format!("{}", e));
            }
        };

    rsx! {
        div { class: "flex items-start justify-between",
            div { class: "flex-1 min-w-0",
                div { class: "flex items-center gap-3 mb-2",
                    h3 { class: "text-sm font-semibold", "{device.name} "
                        span { class: "text-xs font-mono text-zinc-500 mb-2", "{truncated_id}" }
                    }
                }
                if !read_only {
                    button {
                        class: "p-1 text-zinc-500 hover:text-zinc-100 hover:bg-zinc-800 rounded transition-all",
                        onclick: move |_| show_edit_modal.set(true),
                        svg {
                            class: "w-3 h-3",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                d: "M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                            }
                        }
                    }
                }

                if device.iroh_endpoint_id == this_device_id.to_string() {
                    div { class: "text-xs text-zinc-400",
                        "(this device)"
                    }
                }
                else {
                    if let Some(status) = outbound_status {
                        if let Some(current) = status.current {
                            div { class: "text-xs text-zinc-400",
                                "*Active*",
                                if let Some(total) = current.files_total {
                                    "{current.files_transferred}/{total}"
                                }
                                " files"
                            }
                        }
                        if let Some(success) = status.last_success {
                            div { class: "text-xs text-zinc-400",
                                "Last sent {success.files_transferred} file(s) {success.completed_at.relative_time_string()}"
                            }
                        }
                        if let Some(failure) = status.last_failure {
                            div { class: "text-xs text-red-100",
                                "Last failed to send new files {failure.error} {failure.failed_at.relative_time_string()}"
                            }
                        }
                    } else {
                        div { class: "text-xs text-zinc-500",
                            "No outbound mirror record"
                        }
                    }


                    if let Some(status) = inbound_status {
                        if let Some(current) = status.current {
                            div { class: "text-xs text-zinc-400",
                                "*Active*",
                                if let Some(total) = current.files_total {
                                    "{current.files_transferred}/{total}"
                                }
                                " files"
                            }
                        }
                        if let Some(success) = status.last_success {
                            div { class: "text-xs text-zinc-400",
                                "Last recieved {success.files_transferred} file(s) {success.completed_at.relative_time_string()}"
                            }
                        }
                        if let Some(failure) = status.last_failure {
                            div { class: "text-xs text-red-100",
                                "Last failed to receive files {failure.error} {failure.failed_at.relative_time_string()}"
                            }
                        }
                    } else {
                        div { class: "text-xs text-zinc-500",
                            "No incoming mirror record"
                        }
                    }
                }
            }

            if !read_only {
                button {
                    class: "p-2 text-zinc-500 hover:text-red-400 hover:bg-zinc-800 rounded-md transition-all sm:opacity-0 sm:group-hover:opacity-100",
                    onclick: move |_| delete_dialog_open.set(true),
                    svg {
                        class: "w-4 h-4",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            d: "M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                        }
                    }
                }

                if delete_dialog_open() {
                    ConfirmModal {
                        oncancel: move || delete_dialog_open.set(false),
                        onconfirm: delete_device_confirm,
                        p { class: "text-sm text-zinc-300 mb-6",
                            "Deleting this device from your contact record will cease it from syncing with this device. The delete will need to propogate through the system before it is fully deleted. On the other device, transition to standalone to re-join."
                        }
                        if !delete_dialog_error().is_empty() {
                            div { class: "text-sm text-red-400", "{delete_dialog_error}" }
                        }
                    }
                }
            }
        }

        if show_edit_modal() {
            EditDeviceNameModal {
                device_id: device.iroh_endpoint_id.clone(),
                current_name: device.name.clone(),
                oncancel: move |_| show_edit_modal.set(false),
            }
        }

    }
}

#[component]
fn EditDeviceNameModal(device_id: String, current_name: String, oncancel: EventHandler) -> Element {
    let mut app_context = use_context::<AppContext>();
    let mut device_name = use_signal(move || current_name.clone());
    let mut err_message = use_signal(|| String::new());

    let save_device_name = move |_| {
        let vault = app_context.vault.read().clone();
        match vault.device_update(&device_id, device_name.read().as_str()) {
            Ok(_) => {
                if let Err(e) = app_context.reload() {
                    tracing::warn!("failed to reload app: {}", e);
                }
                oncancel.call(());
            }
            Err(e) => {
                err_message.set(format!("Error updating device name: {}", e));
            }
        }
    };

    rsx! {
        div {
            class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50",
            onclick: move |_| oncancel.call(()),

            div {
                class: "w-full max-w-md border border-zinc-700 rounded-lg bg-zinc-900 shadow-2xl p-6",
                onclick: move |evt| evt.stop_propagation(),

                h3 { class: "text-lg font-semibold mb-4", "Edit Device Name" }

                input {
                    class: "w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-md text-sm font-mono focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500 mb-4",
                    r#type: "text",
                    placeholder: "Device name",
                    value: "{device_name}",
                    oninput: move |e| device_name.set(e.value())
                }

                if !err_message().is_empty() {
                    div { class: "mb-4 text-sm text-red-400", "{err_message}" }
                }

                div { class: "flex gap-2 justify-end",
                    button {
                        class: "px-4 py-2 text-sm text-zinc-400 hover:text-zinc-100",
                        onclick: move |_| oncancel.call(()),
                        "Cancel"
                    }
                    button {
                        class: "px-4 py-2 bg-zinc-100 text-zinc-900 rounded-md text-sm font-medium hover:bg-white",
                        onclick: save_device_name,
                        "Save"
                    }
                }
            }
        }
    }
}
