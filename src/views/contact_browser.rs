use crate::model::contact::Contact;
use crate::model::device::Device;
use crate::model::vault::Vault;
use crate::util::sync_status_record::{SyncDirection, SyncStatusRecord};
use crate::AppContext;
use dioxus::prelude::*;

#[component]
pub fn ContactBrowser() -> Element {
    let app_context = use_context::<AppContext>();
    let contact_list = app_context.contacts.read().clone();

    rsx! {
        div { class: "flex flex-col h-full w-2xl",
            ImportComponent {}

            div { class: "space-y-2",
                for contact in contact_list {
                    ContactRow { contact }
                }
            }
        }
    }
}

#[component]
fn ContactRow(contact: Contact) -> Element {
    let mut expanded = use_signal(|| false);

    rsx! {
        div { class: "border border-zinc-800 rounded-lg bg-zinc-900/30 overflow-hidden",
            button { class: "w-full px-6 py-4 hover:bg-zinc-900/50 transition-colors text-left",
                onclick: move |_| expanded.toggle(),
                div { class: "flex items-center justify-between",
                    div { class: "flex-1",
                        div { class: "font-semibold mb-1", "{contact.nickname}" }
                        div { class: "text-sm text-zinc-500", "{contact.username}" }
                    }
                    div { class: "flex items-center gap-4",
                        span { class: "text-sm text-zinc-500", "{contact.devices.len()} devices" }
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
        div { class: "px-6 pb-4 bg-zinc-900/20 border-t border-zinc-800",
            div { class: "space-y-2 pt-4",
                for device in devices {
                    DeviceItem { device }
                }
            }
        }
    }
}

#[component]
fn DeviceItem(device: Device) -> Element {
    let app_context = use_context::<AppContext>();
    let device_for_outbound = device.clone();
    let last_outbound_success = use_signal(move || {
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
    let last_inbound_success = use_signal(move || {
        match SyncStatusRecord::last_success(
            app_context.vault.read().base_path().clone(),
            &device_for_inbound.iroh_endpoint_id,
            SyncDirection::Inbound,
        ) {
            Ok(r) => r,
            Err(_) => None,
        }
    });

    rsx! {
        div { class: "flex items-center justify-between text-sm py-2",
            span { class: "text-zinc-300", "{device.name}"}
            span { class: "text-xs font-mono text-zinc-500 truncate ml-4",
                "{device.iroh_endpoint_id}"
            }
        }
        if let Some(status) = last_outbound_success() {
            div { class: "mt-2 text-xs text-zinc-400",
                "Last outbound sync: ({status.files_transferred} files)" }
        }
        if let Some(status) = last_inbound_success() {
            div { class: "mt-2 text-xs text-zinc-400",
                "Last inbound sync: ({status.files_transferred} files)" }
        }
    }
}

#[component]
fn ImportComponent() -> Element {
    let mut show_modal = use_signal(|| false);
    rsx! {
        section { class: "mb-8",
             button { class: "w-full px-6 py-4 bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 hover:border-zinc-700 rounded-lg text-sm font-medium text-zinc-300 hover:text-zinc-100 transition-all text-left",
                 onclick: move |_| show_modal.set(true),
                 div { class: "flex items-center justify-between",
                     div {
                         div { class: "font-semibold mb-1",
                             "Import Contact Record"
                         }
                         div { class: "text-xs text-zinc-500",
                             "Add a friend to your trust network"
                         }
                     }
                     svg {
                         class: "w-5 h-5 text-zinc-500",
                         fill: "currentColor",
                         view_box: "0 0 20 20",
                         path { d: "M10.75 4.75a.75.75 0 0 0-1.5 0v4.5h-4.5a.75.75 0 0 0 0 1.5h4.5v4.5a.75.75 0 0 0 1.5 0v-4.5h4.5a.75.75 0 0 0 0-1.5h-4.5v-4.5Z" }
                     }
                 }
             }
         }

        if show_modal() {
            ImportModal {
                onclose: move |_| show_modal.set(false)
            }
        }
    }
}

#[component]
fn ImportModal(onclose: EventHandler) -> Element {
    let mut contact_json = use_signal(|| String::new());
    let mut nickname = use_signal(|| String::new());
    let mut err_message = use_signal(|| String::new());
    let mut app_context = use_context::<AppContext>();
    let import_contact = move |_| {
        let vault = app_context.vault.read().clone();
        match vault.contact_import(&nickname.read().clone(), &contact_json.read().clone()) {
            Ok(()) => {
                app_context
                    .contacts
                    .set(vault.contact_read().expect("could not load contacts"));
                onclose.call(());
            }
            Err(e) => err_message.set(format!("Failed to import contact: {e}")),
        };
    };

    rsx! {
        div {
            id: "import-modal",
            class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50",
            div {
                class: "bg-zinc-900 border border-zinc-800 rounded-lg shadow-2xl max-w-2xl w-full h-[90vh] flex flex-col",
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
                    div { class: "flex-3 min-h-0 flex flex-col",
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
