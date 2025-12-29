use crate::model::contact::Contact;
use crate::model::device::Device;
use crate::model::vault::Vault;
use crate::VaultContext;
use dioxus::prelude::*;

#[component]
pub fn ContactBrowser() -> Element {
    let vault_ctx = use_context::<VaultContext>();
    let contacts = use_resource(move || async move {
        let vault_path = vault_ctx.get_vault()?;
        let vault = Vault::new(&vault_path).ok()?;
        vault.contact_read().ok()
    });

    rsx! {
        div { class: "flex flex-col h-full w-2xl",
            ImportComponent {}

            div { class: "flex-1 mt-4",
                match contacts() {
                    Some(Some(contact_list)) => rsx! {
                        for contact in contact_list {
                            ContactRow { contact }
                        }
                    },
                    Some(None) => rsx! {
                        div { "No vault loaded" }
                    },
                    None => rsx! {
                        div { "Loading..." }
                    }
                }
            }
        }
    }
}

#[component]
fn ContactRow(contact: Contact) -> Element {
    let mut expanded = use_signal(|| false);

    rsx! {
        div { class: "flex flex-col border-b",
            button {
                class: "flex items-center justify-between w-full",
                onclick: move |_| expanded.toggle(),
                div { class: "flex flex-col items-start",
                    div { "{contact.nickname}" }
                    div { class: "text-sm opacity-70", "{contact.username}" }
                }
                div { class: "text-xs opacity-50",
                    "{contact.devices.len()} devices"
                }
            }
            if expanded() {
                DeviceItems { devices: contact.devices.clone() }
            }
        }
    }
}

#[component]
fn DeviceItems(devices: Vec<Device>) -> Element {
    rsx! {
        div { class: "flex flex-col",
            for device in devices {
                DeviceItem { device }
            }
        }
    }
}

#[component]
fn DeviceItem(device: Device) -> Element {
    rsx! {
        div { class: "flex justify-between text-sm",
            div { "{device.name}" }
            div { class: "text-xs opacity-50", "{device.iroh_endpoint_id}" }
        }
    }
}

#[component]
fn ImportComponent() -> Element {
    let mut show_modal = use_signal(|| false);
    rsx! {
        div { class: "flex flex-row justify-between",
            button {
                class: "border-1 w-full rounded mt-6",
                r#type: "button",
                onclick: move |_| show_modal.set(true),
                "Import a contact record to enable share with a friend"
            }
            if show_modal() {
                ImportModal {
                    onclose: move |_| show_modal.set(false)
                }
            }
        }
    }
}

#[component]
fn ImportModal(onclose: EventHandler) -> Element {
    let mut contact_json = use_signal(|| String::new());
    let mut nickname = use_signal(|| String::new());
    let mut err_message = use_signal(|| String::new());

    let import_contact = move |_| {
        let vault_ctx = use_context::<VaultContext>();
        let vault_path = vault_ctx.get_vault().expect("vault not set in context!");
        let vault = Vault::new(&vault_path).expect("expecting a local vault");
        match vault.contact_import(&nickname.read().clone(), &contact_json.read().clone()) {
            Ok(()) => onclose.call(()),
            Err(e) => err_message.set(format!("Failed to import contact: {e}")),
        }
    };
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


                        label { "Nickname (the name you want to use when you share with this user):" }
                        input {
                            class: "border-1 px-2",
                            r#type: "text",
                            value: "{nickname}",
                            oninput: move |e| nickname.set(e.value())
                        }
                        label {
                            class: "text-sm",
                            "Paste the contact record below:"
                        }
                        textarea {
                            class: "flex-1 w-full border-1 p-4",
                            value: "{contact_json}",
                            oninput: move |e| contact_json.set(e.value())
                        }
                        label { "{err_message}" }
                        button {
                            r#type: "button",
                            class: "w-full rounded-md bg-indigo-600 px-3 py-2 text-sm font-semibold text-white hover:bg-indigo-500 dark:bg-indigo-500 dark:hover:bg-indigo-400",
                            onclick: import_contact,
                            "Import"
                        }
                        button {
                            r#type: "button",
                            class: "w-full rounded-md bg-indigo-600 px-3 py-2 text-sm font-semibold text-white hover:bg-indigo-500 dark:bg-indigo-500 dark:hover:bg-indigo-400",
                            onclick: move |_| onclose.call(()),
                            "Cancel"
                        }
                    }
                }
            }
        }
    }
}
