use crate::model::vault::VaultState;
use crate::service::join_service::JoinEvent;
use crate::{context::VaultContext, model::vault::Vault};
use crate::{model::user::LocalUser, service::join_service::JoinService};
use dioxus::prelude::*;
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
    let vault_ctx = use_context::<VaultContext>();
    let vault_path = vault_ctx.get_vault().expect("vault not set in context!");
    let vault = Vault::new(&vault_path).expect("expecting a local vault");
    let mut vault_state = use_signal(|| vault.state_read().unwrap_or(VaultState::Uninitialized));

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
                p { "You're using footnote in stand alone mode. Would you like to sync with other devices?" }
                button {
                    class: "border-1 rounded px-2",
                    onclick: move |_|
                        if transition_to_primary().is_ok() {
                            let vault = Vault::new(&vault_path.clone()).expect("expecting a local vault");
                            if let Ok(new_state) = vault.state_read() {
                                vault_state.set(new_state);
                            }
                        },
                    "Make this Primary"
                }
                button {
                    class: "border-1 rounded px-2",
                    onclick: move |_|
                        if transition_to_secondary().is_ok() {
                            if let Ok(new_state) = vault.state_read() {
                                vault_state.set(new_state);
                            }
                        },
                    "Join Existing Vault"
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
    }
}

fn transition_to_primary() -> Result<()> {
    let vault_ctx = use_context::<VaultContext>();
    let vault_path = vault_ctx.get_vault().expect("vault not set in context!");
    let vault = Vault::new(&vault_path).expect("expecting a local vault");
    vault.transition_to_primary("default_username", "default_device_name")?;
    Ok(())
}
fn transition_to_secondary() -> Result<()> {
    let vault_ctx = use_context::<VaultContext>();
    let vault_path = vault_ctx.get_vault().expect("vault not set in context!");
    let vault = Vault::new(&vault_path).expect("expecting a local vault");
    vault.transition_to_secondary("default_device_name")?;
    Ok(())
}

#[component]
fn LocalDeviceComponent(read_only: bool) -> Element {
    let vault_ctx = use_context::<VaultContext>();
    let vault_path = vault_ctx.get_vault().expect("vault not set in context!");
    let vault = Vault::new(&vault_path).expect("expecting a local vault");
    let (_, device_name) = vault.device_public_key().expect("device should exist");
    let mut device_name_value = use_signal(|| device_name);
    let mut err_message = use_signal(|| String::new());
    let save_device_name = move |_| {
        let name_update = device_name_value.read();
        if let Err(e) = vault.device_key_update(&name_update) {
            err_message.set(format!("err updating device name: {}", e));
        }
    };

    rsx! {
        div { class: "grid grid-cols-[auto_1fr_auto] gap-x-2 gap-y-4",
            label { "This Device Name" }
            if read_only {
                span { class: "px-2", "{device_name_value}" }
            } else {
                input {
                    class: "border-1 px-2",
                    r#type: "text",
                    value: "{device_name_value}",
                    oninput: move |e| device_name_value.set(e.value()),
                }
                button {
                    class: "border-1 rounded px-2",
                    onclick: save_device_name,
                    "Update"
                }
            }
        }
        label { "{err_message}" }
    }
}

#[component]
fn UserComponent(read_only: bool) -> Element {
    let vault_ctx = use_context::<VaultContext>();
    let vault_path = vault_ctx.get_vault().expect("vault not set in context!");
    let mut err_message = use_signal(|| String::new());
    let mut username_value = use_signal(|| {
        let vault = Vault::new(&vault_path).expect("expecting a local vault");
        match vault.user_read() {
            Ok(Some(user)) => user.username,
            _ => String::new(),
        }
    });

    let save_username = move |_| {
        let vault = Vault::new(&vault_path).expect("expecting a local vault");
        let username_update = username_value.read();
        if let Err(e) = vault.user_update(&*username_update) {
            err_message.set(format!("err updating username: {}", e));
        }
    };
    rsx! {
        section { class: "border border-zinc-800 rounded-lg bg-zinc-900/30 p-6",
            div { class: "flex items-center gap-4",
                if read_only {
                    span { class: "px-2", "{username_value}" }
                } else {
                    label { "Username" }
                    input {
                        class: "flex-1 px-3 py-2 bg-zinc-900 border border-zinc-700 rounded-md text-sm font-mono focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500",
                        r#type: "text",
                        value: "{username_value}",
                        oninput: move |e| username_value.set(e.value()),
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
    let vault_ctx = use_context::<VaultContext>();
    let vault_path = vault_ctx.get_vault().expect("vault not set in context!");
    let vault = Vault::new(&vault_path).expect("expecting a local vault");
    let devices = vault.device_read()?;
    rsx! {
        div { class: "space-y-8 mt-4",
            section { class: "border border-zinc-800 rounded-lg bg-zinc-900/30 overflow-hidden",
                div { class: "p-6 border-b border-zinc-800",
                    h2 { class: "text-lg font-semibold font-mono", "Devices" }
                    p { class: "text-sm text-zinc-500 mt-1",
                        "\n                            Your connected devices in this vault\n                        "
                    }
                }
                div { class: "divide-y divide-zinc-800",
                    for device in devices.iter() {
                        div { class: "p-6 hover:bg-zinc-900/50 transition-colors group",
                            div { class: "flex items-start justify-between",
                                div { class: "flex-1 min-w-0",
                                    div { class: "flex items-center gap-3 mb-2",
                                        h3 { class: "text-sm font-semibold",
                                            "{device.name}"
                                        }
                                        // span { class: "px-2 py-0.5 bg-zinc-800 border border-zinc-700 rounded text-xs font-mono text-zinc-400",
                                        //     "Primary"
                                        // }
                                    }
                                    p { class: "text-xs font-mono text-zinc-500 truncate",
                                        "{device.iroh_endpoint_id}"
                                    }
                                }
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
    use_effect(move || {
        spawn(async move {
            let vault_ctx = use_context::<VaultContext>();
            let vault_path = vault_ctx.get_vault().expect("vault not set in context!");
            let vault = Vault::new(&vault_path).expect("vault does not exist!");
            match JoinService::listen(&vault, cancel_token()).await {
                Ok(mut rx) => {
                    while let Some(event) = rx.recv().await {
                        match event {
                            JoinEvent::Listening { join_url: url } => join_url.set(url),
                            JoinEvent::Success => onclose.call(()),
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
            class: "fixed inset-0 bg-gray-500/75 dark:bg-gray-900/50 transition-opacity",

            // Centering container
            div {
                class: "flex min-h-full items-end justify-center p-4 text-center sm:items-center sm:p-0",

                // Modal panel
                div {
                    class: "relative transform overflow-hidden rounded-lg bg-white px-4 pt-5 pb-4 text-left shadow-xl transition-all sm:my-8 sm:w-full sm:max-w-sm sm:p-6 dark:bg-gray-800 dark:outline dark:-outline-offset-1 dark:outline-white/10",
                    onclick: move |evt| evt.stop_propagation(),

                    div {
                        div {
                            class: "mt-3 text-center sm:mt-5",
                            h3 {
                                class: "text-base font-semibold text-gray-900 dark:text-white",
                                "Join Device"
                            }
                            div {
                                class: "mt-2",
                                p {
                                    class: "text-sm text-gray-500 dark:text-gray-400",
                                    "Join URL: {join_url}"
                                }
                            }
                            label { "{err_message}" }
                        }
                    }

                    div {
                        class: "mt-5 sm:mt-6",
                        button {
                            r#type: "button",
                            class: "inline-flex w-full justify-center rounded-md bg-indigo-600 px-3 py-2 text-sm font-semibold text-white shadow-xs hover:bg-indigo-500 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 dark:bg-indigo-500 dark:shadow-none dark:hover:bg-indigo-400 dark:focus-visible:outline-indigo-500",
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
        div { class: "flex flex-row justify-between",
            label { "Join a listening device" }
            button {
                class: "rounded-full bg-indigo-600 p-1.5 text-white shadow-xs hover:bg-indigo-500 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 dark:bg-indigo-500 dark:shadow-none dark:hover:bg-indigo-400 dark:focus-visible:outline-indigo-500",
                r#type: "button",
                onclick: move |_| show_modal.set(true),
                svg {
                    class: "size-5",
                    "data-slot": "icon",
                    fill: "currentColor",
                    view_box: "0 0 20 20",
                    path { d: "M10.75 4.75a.75.75 0 0 0-1.5 0v4.5h-4.5a.75.75 0 0 0 0 1.5h4.5v4.5a.75.75 0 0 0 1.5 0v-4.5h4.5a.75.75 0 0 0 0-1.5h-4.5v-4.5Z" }
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

    let join_action = move || {
        spawn(async move {
            let vault_ctx = use_context::<VaultContext>();
            let vault_path = vault_ctx.get_vault().expect("vault not set in context!");
            let vault = Vault::new(&vault_path).expect("expecting a local vault");
            // todo: cancel token
            match JoinService::join(&vault, &join_url()).await {
                Ok(_) => {
                    // Success - close modal or show success
                    onclose.call(());
                }
                Err(e) => {
                    // Show error
                    err_message.set(format!("Join failed: {}", e));
                }
            }
        });
    };

    rsx! {
        div {
            class: "fixed inset-0 bg-gray-500/75 dark:bg-gray-900/50 transition-opacity",
            div {
                class: "flex min-h-full items-end justify-center p-4 text-center sm:items-center sm:p-0",
                div {
                    class: "relative transform overflow-hidden rounded-lg bg-white px-4 pt-5 pb-4 text-left shadow-xl transition-all sm:my-8 sm:w-full sm:max-w-sm sm:p-6 dark:bg-gray-800 dark:outline dark:-outline-offset-1 dark:outline-white/10",
                    onclick: move |evt| evt.stop_propagation(),

                    div {
                        div {
                            class: "mt-3 text-center sm:mt-5",
                            h3 {
                                class: "text-base font-semibold text-gray-900 dark:text-white",
                                "Join Listening Device"
                            }

                            input {
                                class: "border-1 px-2",
                                r#type: "text",
                                oninput: move |e| join_url.set(e.value()),
                            }
                            div {
                                class: "mt-2",
                                "Enter URL"
                            }
                            button {
                                r#type: "button",
                                class: "inline-flex w-full justify-center rounded-md bg-indigo-600 px-3 py-2 text-sm font-semibold text-white shadow-xs hover:bg-indigo-500 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 dark:bg-indigo-500 dark:shadow-none dark:hover:bg-indigo-400 dark:focus-visible:outline-indigo-500",
                                onclick: move |_| join_action(),
                                "Join"
                            }
                            label { "{err_message}" }
                        }
                    }

                    div {
                        class: "mt-5 sm:mt-6",
                        button {
                            r#type: "button",
                            class: "inline-flex w-full justify-center rounded-md bg-indigo-600 px-3 py-2 text-sm font-semibold text-white shadow-xs hover:bg-indigo-500 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 dark:bg-indigo-500 dark:shadow-none dark:hover:bg-indigo-400 dark:focus-visible:outline-indigo-500",
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
fn ExportComponent() -> Element {
    let mut show_modal = use_signal(|| false);
    rsx! {
        div { class: "flex flex-row justify-between",
            button {
                class: "border-1 w-full rounded mt-6",
                r#type: "button",
                onclick: move |_| show_modal.set(true),
                "Create contact record to share with trust network"
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
    let vault_ctx = use_context::<VaultContext>();
    let vault_path = vault_ctx.get_vault().expect("vault not set in context!");
    let vault = Vault::new(&vault_path).expect("expecting a local vault");
    let user_record_json = use_signal(|| match vault.user_read() {
        Ok(Some(user)) => user
            .to_json_pretty()
            .unwrap_or_else(|e| format!("Failed to serialize: {}", e)),
        Ok(None) => "User record not found".to_string(),
        Err(e) => format!("Contact record unable to load: {}", e),
    });
    rsx! {
        div {
            class: "fixed inset-0 bg-gray-500/75 dark:bg-gray-900/50 transition-opacity",

            div {
                class: "flex min-h-full items-center justify-center p-4",

                div {
                    class: "relative w-[90vw] h-[90vh] flex flex-col transform overflow-hidden rounded-lg bg-white shadow-xl dark:bg-gray-800 dark:outline dark:-outline-offset-1 dark:outline-white/10",
                    onclick: move |evt| evt.stop_propagation(),

                    div {
                        class: "p-6 flex flex-col gap-4 flex-1 min-h-0",

                        label {
                            class: "text-sm",
                            "Copy the text below and paste it into an email or text message:"
                        }

                        textarea {
                            class: "flex-1 w-full p-3 text-sm font-mono border rounded-md bg-gray-50 dark:bg-gray-900 dark:border-gray-700 resize-none",
                            readonly: true,
                            value: "{user_record_json}",
                        }

                        button {
                            r#type: "button",
                            class: "w-full rounded-md bg-indigo-600 px-3 py-2 text-sm font-semibold text-white hover:bg-indigo-500 dark:bg-indigo-500 dark:hover:bg-indigo-400",
                            onclick: move |_| onclose.call(()),
                            "Done"
                        }
                    }
                }
            }
        }
    }
}
