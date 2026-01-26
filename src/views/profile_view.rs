use crate::{
    components::{
        app_header::AppHeader,
        app_menu::{AppMenu, MenuButton, MenuDivider},
        confirm_dialog::ConfirmDialog,
        include_device_modal::{IncludeDeviceModalVisible, ListeningDeviceUrl},
    },
    context::AppContext,
    model::{device::Device, vault::VaultState},
    service::join_service::{JoinEvent, JoinService},
};
use dioxus::prelude::*;
use qrcode_generator::QrCodeEcc;
use tokio_util::sync::CancellationToken;

#[component]
pub fn Profile() -> Element {
    let app_context = use_context::<AppContext>();
    let vault_state = app_context.vault_state;

    let mut menu_visible = use_signal(|| false);
    let mut show_edit_username_modal = use_signal(|| false);
    let mut show_export_modal = use_signal(|| false);

    rsx! {
        AppHeader {
            on_menu_click: move |_| menu_visible.set(true),

            h1 {
                class: "flex-1 text-center text-sm font-medium text-zinc-300",
                "Profile"
            }
            div { class: "w-8" }
        }

        AppMenu {
            visible: menu_visible(),
            on_close: move |_| menu_visible.set(false),

            MenuDivider {}

            if matches!(*vault_state.read(), VaultState::Primary) {
                MenuButton {
                    label: "Add Device",
                    onclick: move |_| {
                        consume_context::<ListeningDeviceUrl>().set("".to_string());
                        consume_context::<IncludeDeviceModalVisible>().set(true);
                    }
                }
            }

            if matches!(*vault_state.read(), VaultState::Primary | VaultState::SecondaryJoined) {
                MenuButton {
                    label: "Share Contact Record",
                    onclick: move |_| {
                        show_export_modal.set(true);
                        menu_visible.set(false);
                    }
                }
            }

            if matches!(*vault_state.read(), VaultState::Primary) {
                MenuButton {
                    label: "Edit Username",
                    onclick: move |_| {
                        show_edit_username_modal.set(true);
                        menu_visible.set(false);
                    }
                }
            }

            MenuDivider {}

            MenuButton {
                label: "Debug: Transition to Standalone",
                onclick: move |_| {
                    let mut app_context = use_context::<AppContext>();
                    if app_context.vault.read().transition_to_standalone().is_ok() {
                        if let Err(e) = app_context.reload() {
                            tracing::warn!("failed to reload app: {}", e);
                        }
                    }
                    menu_visible.set(false);
                }
            }
        }

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
                                "You're using Footnote in standalone mode. Would you like to sync with other devices?"
                            }
                            div { class: "flex flex-col sm:flex-row gap-4",
                                TransitionToPrimaryButton{}
                                JoinDeviceGroupButton{}
                            }
                        }
                    },

                    VaultState::SecondaryJoined => rsx! {
                        UserComponent { read_only: true }
                        DeviceListComponent { read_only: true }
                    },

                    VaultState::Primary => rsx! {
                        UserComponent { read_only: true }
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

        if show_export_modal() {
            ExportModal {
                onclose: move |_| show_export_modal.set(false)
            }
        }
    }
}

#[component]
fn EditUsernameModal(oncancel: EventHandler) -> Element {
    let app_context = use_context::<AppContext>();
    let mut username = use_signal(move || match app_context.vault.read().user_read() {
        Ok(Some(user)) => user.username,
        _ => String::new(),
    });
    let mut err_message = use_signal(|| String::new());

    let save_username = move |_| {
        let vault = app_context.vault.read().clone();
        match vault.user_update(username.read().as_str()) {
            Ok(_) => {
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
            "Yes, this my first the device"
        }
    }
}

#[component]
fn JoinDeviceGroupButton() -> Element {
    let mut show_join_interface = use_signal(|| false);

    rsx! {
        if !show_join_interface() {
            button {
                class: "flex-1 px-6 py-3 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 text-zinc-100 rounded-lg font-medium transition-all",
                onclick: move |_| show_join_interface.set(true),
                "Join Device Group"
            }
        } else {
            JoinComponent { oncancel: move |_| show_join_interface.set(false) }
        }
    }
}

#[component]
fn UserComponent(read_only: bool) -> Element {
    let app_context = use_context::<AppContext>();
    let username = use_signal(move || match app_context.vault.read().user_read() {
        Ok(Some(user)) => user.username,
        _ => String::new(),
    });

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
    let device_for_outbound = device.clone();
    let outbound_device_status = use_signal(move || {
        use crate::util::sync_status_record::{SyncDirection, SyncStatusRecord};
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
        use crate::util::sync_status_record::{SyncDirection, SyncStatusRecord};
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

    let truncated_id = truncate_endpoint_id(&device.iroh_endpoint_id);

    rsx! {
        div { class: "flex items-start justify-between",
            div { class: "flex-1 min-w-0",
                div { class: "flex items-center gap-3 mb-2",
                    h3 { class: "text-sm font-semibold", "{device.name} "
                        span { class: "text-xs font-mono text-zinc-500 mb-2", "{truncated_id}" }
                    }
                }

                if let Some(status) = outbound_device_status() {
                    div { class: "text-xs text-zinc-400",
                        "Last outbound: {status.files_transferred} files"
                    }
                }
                if let Some(status) = inbound_device_status() {
                    div { class: "text-xs text-zinc-400",
                        "Last inbound: {status.files_transferred} files"
                    }
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
                            "Deleting this device from your contact record will cease it from syncing with this device. The delete will need to propogate through the system before it is fully deleted. On the other device, transition to standalone to re-join."
                        }
                        if !delete_dialog_error().is_empty() {
                            div { class: "text-sm text-red-400", "{delete_dialog_error}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn JoinComponent(oncancel: EventHandler) -> Element {
    let mut is_listening = use_signal(|| false);
    let mut join_url = use_signal(|| String::new());
    let cancel_token = use_signal(|| CancellationToken::new());
    let mut err_message = use_signal(|| String::new());

    let app_context = use_context::<AppContext>();
    let mut mut_app_context = app_context.clone();

    let img_data = use_memo(move || {
        let svg_data =
            qrcode_generator::to_svg_to_string(join_url(), QrCodeEcc::Medium, 300, None::<&str>)
                .unwrap();
        let safe_svg_data = svg_data.replace(' ', "%20").replace('#', "%23");
        format!("data:image/svg+xml;utf8,{}", safe_svg_data)
    });

    let start_listening = move |_| {
        is_listening.set(true);
        err_message.set(String::new());

        let vault = app_context.vault.read().clone();
        spawn(async move {
            match JoinService::listen(&vault, cancel_token()).await {
                Ok(mut rx) => {
                    while let Some(event) = rx.recv().await {
                        match event {
                            JoinEvent::Listening { join_url: url } => {
                                tracing::info!("received join url for disply {}", join_url);
                                join_url.set(url)
                            }
                            JoinEvent::Success => {
                                tracing::info!("join success");
                                if let Err(e) = mut_app_context.reload() {
                                    tracing::warn!("failed to reload app: {}", e);
                                } else {
                                    tracing::info!("reloaded app");
                                }
                                return;
                            }
                            JoinEvent::Error(e) => err_message.set(format!("{}", e)),
                        }
                    }
                }
                Err(e) => err_message.set(format!("{}", e)),
            }
        });
    };

    let cancel_listening = move |_| {
        cancel_token().cancel();
        oncancel.call(());
    };

    use_drop(move || {
        if is_listening() {
            cancel_token().cancel();
        }
    });

    rsx! {
        div {
            class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50",
            onclick: move |_| oncancel.call(()),

            div {
                class: "bg-zinc-900 border border-zinc-800 rounded-lg shadow-2xl max-w-md w-full",
                onclick: move |evt| evt.stop_propagation(),

                div { class: "p-6 border-b border-zinc-800",

                    if !is_listening() {
                        div {
                            p { class: "text-sm text-zinc-500 mb-4",
                                "Connect this device to your primary device"
                            }
                            div { class: "flex gap-3",
                                button {
                                    class: "px-6 py-3 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-lg font-medium transition-all",
                                    onclick: move |_| oncancel.call(()),
                                    "Cancel"
                                }
                                button {
                                    class: "px-6 py-3 bg-zinc-100 hover:bg-white text-zinc-900 rounded-lg font-medium transition-all",
                                    onclick: start_listening,
                                    "Listen for Primary Device"
                                }
                            }
                        }
                    } else {
                        div {
                            p { class: "text-sm text-zinc-500 mb-6",
                                "Scan this QR code on your primary device to complete pairing"
                            }

                            div { class: "flex flex-col items-center mb-6",
                                div { class: "bg-white p-4 rounded-lg mb-4",
                                    div { class: "w-48 h-48 bg-zinc-200 flex items-center justify-center text-zinc-500 text-xs",
                                        img {
                                            width: 300,
                                            height: 300,
                                            class: "transition-opacity duration-300",
                                            style: "image-rendering: pixelated;",
                                            src: "{img_data}"
                                        }
                                    }
                                }

                                div { class: "w-full",
                                    label { class: "block text-xs font-medium text-zinc-400 mb-2",
                                        "Join URL"
                                    }
                                    div { class: "bg-zinc-950 border border-zinc-800 rounded-lg p-3",
                                        p { class: "select-all break-all font-mono text-xs text-zinc-400",
                                            "{join_url}"
                                        }
                                    }
                                }
                            }

                            if !err_message().is_empty() {
                                div { class: "mb-4 p-3 bg-red-900/20 border border-red-800 rounded-lg text-sm text-red-400",
                                    "{err_message}"
                                }
                            }

                            button {
                                class: "px-4 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all",
                                onclick: cancel_listening,
                                "Cancel"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ExportModal(onclose: EventHandler) -> Element {
    let app_context = use_context::<AppContext>();
    let mut error_message = use_signal(|| None::<String>);

    let user_record_json = use_signal(move || match app_context.vault.read().user_read() {
        Ok(Some(user)) => user
            .to_json_pretty()
            .unwrap_or_else(|e| format!("Failed to serialize: {}", e)),
        Ok(None) => "User record not found".to_string(),
        Err(e) => format!("Contact record unable to load: {}", e),
    });

    let handle_share = move |_| {
        use crate::platform;
        error_message.set(None);

        let Some((username, timestamp, Some(json_content))) = app_context
            .vault
            .read()
            .user_read()
            .ok()
            .flatten()
            .map(|u| (u.username.clone(), u.updated_at, u.to_json_pretty().ok()))
        else {
            error_message.set(Some("failed to get user record".into()));
            return;
        };

        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!("{}.{}.fncontact", username, timestamp));

        match std::fs::write(&file_path, json_content) {
            Ok(_) => {
                if let Err(e) = platform::share_contact_file(&file_path) {
                    error_message.set(Some(format!("Share failed: {}", e)));
                }
            }
            Err(e) => {
                error_message.set(Some(format!("Failed to create file: {}", e)));
            }
        }
    };

    rsx! {
        div {
            class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50",
            div {
                class: "bg-zinc-900 border border-zinc-800 rounded-lg shadow-2xl w-full max-w-2xl h-[80vh] flex flex-col",
                onclick: move |evt| evt.stop_propagation(),
                div { class: "p-6 border-b border-zinc-800",
                    h3 { class: "text-lg font-semibold font-mono", "Export Contact Record" }
                    p { class: "text-sm text-zinc-500 mt-1",
                        if cfg!(target_os = "android") {
                            "Share or copy this with your trusted contacts"
                        } else {
                            "Copy and share this with your trusted contacts"
                        }
                    }
                }
                div { class: "p-6 flex-1 min-h-0 flex flex-col",
                    textarea {
                        class: "flex-1 w-full select-all px-4 py-3 bg-zinc-950 border border-zinc-800 rounded-lg text-xs font-mono text-zinc-300 resize-none focus:border-zinc-600 focus:ring-1 focus:ring-zinc-600 mb-4",
                        readonly: "true",
                        "{user_record_json}"
                    }

                    if let Some(error) = error_message() {
                        div { class: "mb-4 p-3 bg-red-900/20 border border-red-800 rounded-lg text-sm text-red-400",
                            "{error}"
                        }
                    }

                    {
                        use crate::platform;
                        if platform::SHARE_SHEET_SUPPORTED {
                            rsx! {
                                div { class: "flex gap-3",
                                    button {
                                        class: "flex-1 px-4 py-2 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 rounded-md text-sm font-medium transition-all",
                                        onclick: handle_share,
                                        "Share"
                                    }
                                    button {
                                        class: "flex-1 px-4 py-2 bg-zinc-300 hover:bg-white text-zinc-900 rounded-md text-sm font-medium transition-all",
                                        onclick: move |_| onclose.call(()),
                                        "Done"
                                    }
                                }
                            }
                        } else {
                            rsx! {
                                button {
                                    class: "w-full px-4 py-2 bg-zinc-300 hover:bg-white text-zinc-900 rounded-md text-sm font-medium transition-all",
                                    onclick: move |_| onclose.call(()),
                                    "Done"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
