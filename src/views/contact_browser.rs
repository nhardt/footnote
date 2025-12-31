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
            id: "import-modal",
            class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50",
            div {
                class: "bg-zinc-900 border border-zinc-800 rounded-lg shadow-2xl max-w-2xl w-full max-h-[90vh] flex flex-col",
                onclick: move |evt| evt.stop_propagation(),
                div { class: "p-6 border-b border-zinc-800",
                    h3 { class: "text-lg font-semibold font-mono",
                        "Import Contact"
                    }
                    p { class: "text-sm text-zinc-500 mt-1",
                        "Add someone to your trust network"
                    }
                }
                div { class: "p-6 flex-1 min-h-0 flex flex-col gap-4",
                    div {
                        label { class: "block text-sm font-medium text-zinc-300 mb-2",
                            "Nickname"
                            span { class: "text-zinc-500 font-normal ml-1",
                                "(how you'll reference them when sharing)"
                            }
                        }
                        input {
                            class: "w-full px-3 py-2 bg-zinc-950 border border-zinc-700 rounded-md text-sm font-mono focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500",
                            placeholder: "alice",
                            r#type: "text",
                            value: "{nickname}",
                            oninput: move |e| nickname.set(e.value())
                        }
                    }
                    div { class: "flex-1 min-h-0 flex flex-col",
                        label { class: "block text-sm font-medium text-zinc-300 mb-2",
                            "Contact Record"
                        }
                        textarea {
                            class: "flex-1 w-full px-4 py-3 bg-zinc-950 border border-zinc-700 rounded-lg text-xs font-mono text-zinc-300 resize-none focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500",
                            value: "{contact_json}",
                            oninput: move |e| contact_json.set(e.value())
                        }
                    }
                    div {
                        class: "text-sm text-red-400 font-mono",
                        style: "display: none",
                        "{err_message}"
                    }
                    div { class: "flex gap-3",
                        button { class: "flex-1 px-4 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all",
                            onclick: move |_| onclose.call(()),
                            "Cancel"
                        }
                        button { class: "flex-1 px-4 py-2 bg-zinc-100 hover:bg-white hover:shadow-lg text-zinc-900 rounded-md text-sm font-medium transition-all",
                            onclick: import_contact,
                            "Import"
                        }
                    }
                }
            }
        }
    }
}
