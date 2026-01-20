use crate::components::import_contact_modal::ImportContactModal;
use crate::context::ImportContactModalVisible;
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
                 onclick: move |_| consume_context::<ImportContactModalVisible>().set(true),
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
    }
}
