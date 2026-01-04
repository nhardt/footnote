use crate::components::confirm_dialog::ConfirmDialog;
use crate::context::AppContext;
use crate::model::device::Device;
use crate::model::vault::Vault;
use crate::model::vault::VaultState;
use crate::service::join_service::JoinEvent;
use crate::util::sync_status_record::{SyncDirection, SyncStatusRecord};
use crate::{model::user::LocalUser, service::join_service::JoinService};
use dioxus::prelude::*;
use std::collections::HashMap;
use tokio_util::sync::CancellationToken;

// Elements of a user profile by VaultState:
// - StandAlone:
//   - no user profile, you just have notes on disk
// - SecondaryUnjoined:
//   - has a device_key, created on first join attempt
// - SecondaryJoined:
//   - has a device_key, has a user.json (from primary)
// - Primary:
//   - has a device_key, a user.json and signing key (id_key)
//
// This form will be sort of like a workflow. Each step will reveal the next step
//
// VaultState: [ StandAlone ]
// [ Create Primary -> Primary ][ Join Primary -> SecondaryUnjoined ]
//
// VaultState: [ Primary ]
// [ Username Control ]
// [ Device Name Control ]
// {Devices}
// [ + Add Device + ] -> Device Join Listen Modal
//
// VaultState: [ SecondaryUnjoined ]
// {DeviceName}
// [ Join ] -> Device Join Modal (join url) -> SecondaryJoined
//
// VaultState: [ SecondaryJoined ]
// {DeviceName}

#[component]
pub fn Profile() -> Element {
    let app_context = use_context::<AppContext>();
    let vault_state = app_context.vault_state;

    rsx! {
        div { class: "mb-12",
            h1 { class: "text-3xl font-bold font-mono mb-2",
                "Profile: "
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
                div { id: "standalone-state",
                    main { class: "max-w-3xl mx-auto px-6 py-12",
                        div { class: "mb-12",
                            h1 { class: "text-3xl font-bold font-mono mb-2",
                                "Vault: "
                                span { class: "text-zinc-400", "Standalone" }
                            }
                            p { class: "text-sm text-zinc-500",
                                "Your vault is ready for local use"
                            }
                        }
                        div { class: "border border-zinc-800 rounded-lg bg-zinc-900/30 p-8",
                            p { class: "text-zinc-300 mb-8",
                                "You're using footnote in standalone mode. Would you like to sync with other devices?"
                            }
                            div { class: "flex gap-4",
                                TransitionToPrimaryButton{}
                                TransitionToSecondaryButton{}
                            }
                        }
                    }
                }
            },

            VaultState::SecondaryUnjoined => rsx! {
                LocalDeviceComponent { read_only: false }
                JoinComponent{}
            },

            VaultState::SecondaryJoined => rsx! {
                UserComponent { read_only: true }
                DeviceListComponent { read_only: true }
                ExportComponent {}
            },

            VaultState::Primary => rsx! {
                UserComponent { read_only: false }
                DeviceListComponent { read_only: false }
                ExportComponent {}
            }
        }

        TransitionToStandAloneButton{}
    }
}

#[component]
fn TransitionToPrimaryButton() -> Element {
    let mut app_context = use_context::<AppContext>();

    let onclick = move |_| {
        if app_context
            .vault
            .read()
            .transition_to_primary("default", "primary")
            .is_ok()
        {
            if let Err(e) = app_context.reload() {
                tracing::warn!("failed to reload app: {}", e);
            }
        }
    };

    rsx! {
        button {
            class: "flex-1 px-6 py-3 bg-zinc-100 hover:bg-white text-zinc-900 rounded-lg font-medium transition-all",
            onclick: onclick,
            "Make This Primary"
        }
    }
}

#[component]
fn TransitionToSecondaryButton() -> Element {
    let mut app_context = use_context::<AppContext>();

    let onclick = move |_| {
        if app_context
            .vault
            .read()
            .transition_to_secondary("secondary")
            .is_ok()
        {
            if let Err(e) = app_context.reload() {
                tracing::warn!("failed to reload app: {}", e);
            }
        }
    };

    rsx! {
        button { class: "flex-1 px-6 py-3 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 text-zinc-100 rounded-lg font-medium transition-all",
            onclick: onclick,
            "Join Existing Vault"
        }
    }
}

#[component]
fn TransitionToStandAloneButton() -> Element {
    let mut app_context = use_context::<AppContext>();
    let onclick = move |_| {
        if app_context.vault.read().transition_to_standalone().is_ok() {
            if let Err(e) = app_context.reload() {
                tracing::warn!("failed to reload app: {}", e);
            }
        }
    };

    rsx! {
        button {
            class: "border-1 mt-16",
            onclick: onclick,
            "Debug: Transition To Standalone"
        }
    }
}

#[component]
fn LocalDeviceComponent(read_only: bool) -> Element {
    let app_context = use_context::<AppContext>();
    let mut device_name = use_signal(move || {
        let (_, device_name) = app_context
            .vault
            .read()
            .device_public_key()
            .expect("device should exist");
        device_name
    });

    let mut err_message = use_signal(|| String::new());
    let save_device_name = move |_| {
        let name_update = device_name.read();
        if let Err(e) = app_context.vault.read().device_key_update(&name_update) {
            err_message.set(format!("err updating device name: {}", e));
        }
    };

    rsx! {
        div { class: "border border-zinc-800 rounded-lg bg-zinc-900/30 p-6",
            div { class: "flex items-center gap-4",
                label { class: "text-sm font-medium text-zinc-300 w-32", "This Device Name" }
                if read_only {
                    span { class: "flex-1 px-3 py-2 text-sm font-mono text-zinc-300", "{device_name}" }
                } else {
                    input {
                        class: "flex-1 px-3 py-1.5 bg-zinc-900 border border-zinc-700 rounded-md text-sm font-mono focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500",
                        r#type: "text",
                        value: "{device_name}",
                        oninput: move |e| device_name.set(e.value()),
                    }
                    button {
                        class: "px-4 py-1.5 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all",
                        onclick: save_device_name,
                        "Update"
                    }
                }
            }
            if !err_message().is_empty() {
                div { class: "mt-2 text-sm text-red-400 font-mono", "{err_message}" }
            }
        }
    }
}

#[component]
fn UserComponent(read_only: bool) -> Element {
    let app_context = use_context::<AppContext>();

    let mut err_message = use_signal(|| String::new());
    let mut username = use_signal(move || match app_context.vault.read().user_read() {
        Ok(Some(user)) => user.username,
        _ => String::new(),
    });

    let save_username = move |_| {
        let vault = app_context.vault.read().clone();
        if let Err(e) = vault.user_update(username.read().as_str()) {
            err_message.set(format!("err updating username: {}", e));
        }
    };
    rsx! {
        section { class: "border border-zinc-800 rounded-lg bg-zinc-900/30 p-6",
            div { class: "flex items-center gap-4",
                if read_only {
                    span { class: "px-2", "{username}" }
                } else {
                    label { "Username" }
                    input {
                        class: "flex-1 px-3 py-2 bg-zinc-900 border border-zinc-700 rounded-md text-sm font-mono focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500",
                        r#type: "text",
                        value: "{username}",
                        oninput: move |e| username.set(e.value()),
                    }
                    button { class: "px-4 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all",
                        onclick: save_username,
                        "Update"
                    }
                }
            }
            label { "{err_message}" }
        }
    }
}

#[component]
fn DeviceListComponent(read_only: bool) -> Element {
    let app_context = use_context::<AppContext>();
    let devices = app_context.devices.clone();
    rsx! {
        div { class: "space-y-8 mt-4",
            section { class: "border border-zinc-800 rounded-lg bg-zinc-900/30 overflow-hidden",
                div { class: "p-6 border-b border-zinc-800",
                    h2 { class: "text-lg font-semibold font-mono", "Devices" }
                    p { class: "text-sm text-zinc-500 mt-1",
                        "Your connected devices in this vault"
                    }
                }
                div { class: "divide-y divide-zinc-800",
                    for device in devices.iter() {
                        div { class: "p-6 hover:bg-zinc-900/50 transition-colors group",
                            div { class: "flex items-start justify-between",
                                DeviceRow{ device:device.clone(), read_only: read_only }
                            }
                        }
                    }
                }
                if read_only {
                    div { class: "p-6 bg-zinc-900/20 border-t border-zinc-800",
                        button { class: "flex items-center gap-3 text-sm font-medium text-zinc-300 hover:text-zinc-100 transition-colors group",
                            div { class: "p-1.5 rounded-full bg-zinc-800 group-hover:bg-zinc-700 border border-zinc-700 group-hover:border-zinc-600 transition-all",
                            }
                            span { "Manage additional devices on primary" }
                        }
                    }
                }
                else {
                    JoinListenerComponent {}
                }
            }
        }
    }
}

#[component]
fn DeviceRow(device: Device, read_only: bool) -> Element {
    let mut app_context = use_context::<AppContext>();
    let device_for_outbound = device.clone();
    let outbound_device_status = use_signal(move || {
        match SyncStatusRecord::last_success(
            app_context.vault.read().base_path().clone(),
            &device_for_outbound.iroh_endpoint_id,
            SyncDirection::Outbound,
        ) {
            Ok(r) => r,
            Err(_) => None,
        }
    });

    let device_for_inbound = device.clone();
    let inbound_device_status = use_signal(move || {
        match SyncStatusRecord::last_success(
            app_context.vault.read().base_path().clone(),
            &device_for_inbound.iroh_endpoint_id,
            SyncDirection::Inbound,
        ) {
            Ok(r) => r,
            Err(_) => None,
        }
    });

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
        div { class: "flex-1 min-w-0",
            div { class: "flex items-center gap-3 mb-2",
                h3 { class: "text-sm font-semibold",
                    "{device.name}"
                }
                if let Some(status) = outbound_device_status() {
                    div { class: "mt-2 text-xs text-zinc-400",
                        "Last outbound sync: ({status.files_transferred} files)" }
                }
                if let Some(status) = inbound_device_status() {
                    div { class: "mt-2 text-xs text-zinc-400",
                        "Last inbound sync: ({status.files_transferred} files)" }
                }
                // span { class: "px-2 py-0.5 bg-zinc-800 border border-zinc-700 rounded text-xs font-mono text-zinc-400",
                //     "Primary"
                // }
            }
            p { class: "text-xs font-mono text-zinc-500 truncate",
                "{device.iroh_endpoint_id}"
            }
        }
        if !read_only {
            button {
                class: "opacity-0 group-hover:opacity-100 p-2 text-zinc-500 hover:text-red-400 hover:bg-zinc-800 rounded-md transition-all",
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
                ConfirmDialog {
                    oncancel: move || delete_dialog_open.set(false),
                    onconfirm: delete_device_confirm,
                    p { class: "text-sm text-zinc-300 mb-6",
                        "Deleting this device from your contact record will cease it from syncing with this device. The delete will need to propogate through the system before it is fully deleted. On the other device, transition to standalone to re-join.",
                    }
                    label { {delete_dialog_error} }
                }
            }
        }
    }
}

#[component]
fn JoinListenerComponent() -> Element {
    // join is a 3 step process:
    // - create join listener on primary, return join_code
    // - OOB transfer of join_code to secondary
    // - join(join_code) on secondary
    //
    // on the primary, since this is rare and probably needs to "user
    // synchonous", we'll just make this a modal
    let mut show_modal = use_signal(|| false);
    rsx! {
        div { class: "p-6 bg-zinc-900/20 border-t border-zinc-800",
            button { class: "flex items-center gap-3 text-sm font-medium text-zinc-300 hover:text-zinc-100 transition-colors group",
                r#type: "button",
                onclick: move |_| show_modal.set(true),
                div { class: "p-1.5 rounded-full bg-zinc-800 group-hover:bg-zinc-700 border border-zinc-700 group-hover:border-zinc-600 transition-all",
                    svg {
                        class: "w-4 h-4",
                        fill: "currentColor",
                        view_box: "0 0 20 20",
                        path { d: "M10.75 4.75a.75.75 0 0 0-1.5 0v4.5h-4.5a.75.75 0 0 0 0 1.5h4.5v4.5a.75.75 0 0 0 1.5 0v-4.5h4.5a.75.75 0 0 0 0-1.5h-4.5v-4.5Z" }
                    }
                }
                span { "Join a device you own to this vault" }
                if show_modal() {
                    JoinListenModal {
                        onclose: move |_| show_modal.set(false)
                    }
                }
            }
        }
    }
}

#[component]
fn JoinListenModal(onclose: EventHandler) -> Element {
    let mut join_url = use_signal(|| String::new());
    let cancel_token = use_signal(|| CancellationToken::new());
    let mut err_message = use_signal(|| String::new());

    // Start listening on mount
    let app_context = use_context::<AppContext>();
    let mut mut_app_context = app_context.clone();
    use_effect(move || {
        let vault = app_context.vault.read().clone();
        spawn(async move {
            match JoinService::listen(&vault, cancel_token()).await {
                Ok(mut rx) => {
                    while let Some(event) = rx.recv().await {
                        match event {
                            JoinEvent::Listening { join_url: url } => join_url.set(url),
                            JoinEvent::Success => {
                                if let Err(e) = mut_app_context.reload() {
                                    tracing::warn!("failed to reload app: {}", e);
                                }
                                onclose.call(())
                            }
                            JoinEvent::Error(e) => err_message.set(format!("{}", e)),
                        }
                    }
                }
                Err(e) => err_message.set(format!("{}", e)),
            }
        });
    });

    // Cancel on unmount
    use_drop(move || cancel_token().cancel());

    rsx! {
        div {
            class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50",
            onclick: move |evt| evt.stop_propagation(),
            div {
                class: "bg-zinc-900 border border-zinc-800 rounded-lg shadow-2xl max-w-md w-full",
                div { class: "p-6 border-b border-zinc-800",
                    h3 { class: "text-lg font-semibold font-mono", "Join Device" }
                    p { class: "text-sm text-zinc-500 mt-1",
                        "Share this URL with your secondary device"
                    }
                }
                div { class: "p-6",
                    div { class: "bg-zinc-950 border border-zinc-800 rounded-lg p-4 mb-6",
                        p { class: "select-all break-all font-mono text-zinc-400",
                            "{join_url}"
                        }
                    }
                    label { "{err_message}" }
                    div { class: "flex gap-3",
                        // button { class: "flex-1 px-4 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all",
                        //     "Copy URL"
                        // }
                        div { class: "flex-1 px-4 py-2",
                            ""
                        }
                        button { class: "px-4 py-2 bg-zinc-100 hover:bg-white text-zinc-900 rounded-md text-sm font-medium transition-all",
                            onclick: move |_| onclose.call(()),
                            "Cancel"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn JoinComponent() -> Element {
    let mut show_modal = use_signal(|| false);
    rsx! {
        div { class: "border border-zinc-800 rounded-lg bg-zinc-900/30 overflow-hidden mt-8",
            div { class: "p-6 bg-zinc-900/20",
                button {
                    class: "flex items-center gap-3 text-sm font-medium text-zinc-300 hover:text-zinc-100 transition-colors group",
                    onclick: move |_| show_modal.set(true),
                    div { class: "p-1.5 rounded-full bg-zinc-800 group-hover:bg-zinc-700 border border-zinc-700 group-hover:border-zinc-600 transition-all",
                        svg {
                            class: "w-4 h-4",
                            fill: "currentColor",
                            view_box: "0 0 20 20",
                            path { d: "M10.75 4.75a.75.75 0 0 0-1.5 0v4.5h-4.5a.75.75 0 0 0 0 1.5h4.5v4.5a.75.75 0 0 0 1.5 0v-4.5h4.5a.75.75 0 0 0 0-1.5h-4.5v-4.5Z" }
                        }
                    }
                    span { "Join a listening device" }
                }
            }
            if show_modal() {
                JoinModal {
                    onclose: move |_| show_modal.set(false)
                }
            }
        }
    }
}

#[component]
fn JoinModal(onclose: EventHandler) -> Element {
    let cancel_token = use_signal(|| CancellationToken::new());
    let mut err_message = use_signal(|| String::new());
    let mut join_url = use_signal(|| String::new());

    use_drop(move || cancel_token().cancel());

    let app_context = use_context::<AppContext>();
    let mut mut_app_context = use_context::<AppContext>();
    let join_action = move || {
        let vault = app_context.vault.read().clone();
        spawn(async move {
            // todo: cancel token
            match JoinService::join(&vault, &join_url()).await {
                Ok(_) => {
                    if let Err(e) = mut_app_context.reload() {
                        tracing::warn!("failed to reload app: {}", e);
                    }
                    onclose.call(());
                }
                Err(e) => {
                    err_message.set(format!("Join failed: {}", e));
                }
            }
        });
    };

    rsx! {
        div {
            class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50",
            id: "join-modal",
            div {
                class: "bg-zinc-900 border border-zinc-800 rounded-lg shadow-2xl max-w-md w-full",
                onclick: move |evt| evt.stop_propagation(),
                div { class: "p-6 border-b border-zinc-800",
                    h3 { class: "text-lg font-semibold font-mono",
                        "Join Listening Device"
                    }
                    p { class: "text-sm text-zinc-500 mt-1",
                        "Enter the join URL from your primary device"
                    }
                }
                div { class: "p-6",
                    div { class: "mb-6",
                        label { class: "block text-sm font-medium text-zinc-300 mb-2",
                            "Join URL"
                        }
                        input {
                            class: "w-full px-3 py-2 bg-zinc-950 border border-zinc-800 rounded-md text-sm font-mono focus:border-zinc-600 focus:ring-1 focus:ring-zinc-600",
                            placeholder: "iroh://...",
                            r#type: "text",
                            oninput: move |e| join_url.set(e.value())
                        }
                    }
                    div { class: "flex gap-3",
                        button { class: "flex-1 px-4 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all",
                            onclick: move |_| onclose.call(()),
                            "Cancel"
                        }
                        button { class: "flex-1 px-4 py-2 bg-zinc-100 hover:bg-white text-zinc-900 rounded-md text-sm font-medium transition-all",
                            onclick: move |_| join_action(),
                            "Join"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ExportComponent() -> Element {
    let mut show_modal = use_signal(|| false);
    rsx! {
        section { class: "pt-4",
            button { class: "w-full px-6 py-4 bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 hover:border-zinc-700 rounded-lg text-sm font-medium text-zinc-300 hover:text-zinc-100 transition-all text-left",
                onclick: move |_| show_modal.set(true),
                div { class: "flex items-center justify-between",
                    div {
                        div { class: "font-semibold mb-1",
                            "Create Contact Record"
                        }
                        div { class: "text-xs text-zinc-500",
                            "Share with your trust network"
                        }
                    }
                    svg {
                        class: "w-5 h-5 text-zinc-500",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            d: "M9 5l7 7-7 7",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                        }
                    }
                }
            }
            if show_modal() {
                ExportModal {
                    onclose: move |_| show_modal.set(false)
                }
            }
        }
    }
}

#[component]
fn ExportModal(onclose: EventHandler) -> Element {
    let app_context = use_context::<AppContext>();
    let user_record_json = use_signal(move || match app_context.vault.read().user_read() {
        Ok(Some(user)) => user
            .to_json_pretty()
            .unwrap_or_else(|e| format!("Failed to serialize: {}", e)),
        Ok(None) => "User record not found".to_string(),
        Err(e) => format!("Contact record unable to load: {}", e),
    });
    rsx! {
        div {
            id: "export-modal",
            class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50",
            div {
                class: "bg-zinc-900 border border-zinc-800 rounded-lg shadow-2xl max-w-2xl w-full h-[90vh] flex flex-col",
                onclick: move |evt| evt.stop_propagation(),
                div { class: "p-6 border-b border-zinc-800",
                    h3 { class: "text-lg font-semibold font-mono",
                        "Export Contact Record"
                    }
                    p { class: "text-sm text-zinc-500 mt-1",
                        "Copy and share this with your trusted contacts"
                    }
                }
                div { class: "p-6 flex-1 min-h-0 flex flex-col",
                    textarea {
                        class: "flex-1 w-full select-all px-4 py-3 bg-zinc-950 border border-zinc-800 rounded-lg text-xs font-mono text-zinc-300 resize-none focus:border-zinc-600 focus:ring-1 focus:ring-zinc-600 mb-4",
                        readonly: "true",
                        "{user_record_json}"
                    }
                    button { class: "w-full px-4 py-2 bg-zinc-300 hover:bg-white text-zinc-900 rounded-md text-sm font-medium transition-all",
                        onclick: move |_| onclose.call(()),
                        "Done"
                    }
                }
            }
        }
    }
}
