use crate::model::vault::VaultState;
use crate::service::join_service::JoinEvent;
use crate::{context::VaultContext, model::vault::Vault};
use crate::{model::user::LocalUser, service::join_service::JoinService};
use dioxus::prelude::*;
use dioxus_clipboard::prelude::use_clipboard;
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
    let state = vault.state_read()?;

    rsx! {
        div { class: "flex flex-col h-full w-2xl gap-2",
            h2 { class: "text-2xl font-bold", "Profile" }

            match state {
                VaultState::Uninitialized => rsx! {
                    p { "Vault not initialized" }
                },

                VaultState::StandAlone => rsx! {
                    p { "You're using footnote in stand alone mode. Would you like to sync with other devices?" }
                    button {
                        class: "border-1 rounded px-2",
                        onclick: move |_|  transition_to_primary(),
                        "Make this Primary"
                    }
                    button {
                        class: "border-1 rounded px-2",
                        onclick: move |_| transition_to_unjoined(),
                        "Join Existing Vault"
                    }
                },

                VaultState::SecondaryUnjoined => rsx! {
                    LocalDeviceComponent { read_only: false }
                    //JoinModalButton {}
                },

                VaultState::SecondaryJoined => rsx! {
                    UserComponent { read_only: true }
                    DeviceListComponent { read_only: true }
                },

                VaultState::Primary => rsx! {
                    UserComponent { read_only: false }
                    DeviceListComponent { read_only: false }
                    JoinListenerComponent {}
                }
            }
        }
    }
}

fn transition_to_primary() -> Result<()> {
    let vault_ctx = use_context::<VaultContext>();
    let vault_path = vault_ctx.get_vault().expect("vault not set in context!");
    let vault = Vault::new(&vault_path).expect("expecting a local vault");
    vault.transition_to_primary("default", "default")?;

    Ok(())
}
fn transition_to_unjoined() {}

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
    let vault = Vault::new(&vault_path).expect("expecting a local vault");
    let mut err_message = use_signal(|| String::new());
    let mut username_value = use_signal(|| String::new());

    let save_username = move |_| {
        let vault = Vault::new(&vault_path).expect("expecting a local vault");
        let username_update = username_value.read();
        if let Err(e) = vault.user_update(&*username_update) {
            err_message.set(format!("err updating username: {}", e));
        }
    };
    match vault.user_read() {
        Ok(Some(user)) => {
            username_value.set(user.username);
            rsx! {
                div { class: "flex flex-row gap-x-2 gap-y-4",
                    if read_only {
                        span { class: "px-2", "{username_value}" }
                    } else {
                        input {
                            class: "border-1 px-2",
                            r#type: "text",
                            value: "{username_value}",
                            oninput: move |e| username_value.set(e.value()),
                        }
                        button {
                            class: "border-1 rounded px-2",
                            onclick: save_username,
                            "Update"
                        }
                    }
                }
                label { "{err_message}" }
            }
        }
        Ok(None) => rsx! {
            p { "No user record" }
        },
        Err(e) => rsx! {
            p { class: "text-red-600", "Error: {e}" }
        },
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
        div { class: "flex flex-row justify-between",
            label { "Join a device you own to this vault" }
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
fn DeviceListComponent(read_only: bool) -> Element {
    let vault_ctx = use_context::<VaultContext>();
    let vault_path = vault_ctx.get_vault().expect("vault not set in context!");
    let vault = Vault::new(&vault_path).expect("expecting a local vault");
    let devices = vault.device_read()?;
    rsx! {
        h2 { class: "text-2xl font-bold", "Devices" }
        div { class: "grid grid-cols-2",
            for device in devices.iter() {
                span { "{device.name}" }
                span { class: "truncate", "{device.iroh_endpoint_id}" }
            }
        }
    }
}

async fn copy_to_clipboard(txt: &str) -> anyhow::Result<()> {
    let mut clipboard = use_clipboard();
    let _ = clipboard.set(txt.to_string());
    Ok(())
}
