use dioxus::prelude::*;

use footnote_core::model::contact::Contact;
use footnote_core::model::device::Device;
use footnote_core::util::sync_status_record::{SyncDirection, SyncStatusRecord};

use crate::context::AppContext;

#[component]
pub fn ContactBrowser() -> Element {
    let app_context = use_context::<AppContext>();
    let contact_list = app_context.contacts.read().clone();

    rsx! {
        main { class: "flex-1 overflow-y-auto",
            div { class: "max-w-3xl mx-auto px-4 py-6 sm:px-6",
                div { class: "space-y-2",
                    for contact in contact_list {
                        ContactRow { contact }
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
        div { class: "border border-zinc-800 text-zinc-100 rounded-lg bg-zinc-900/30 overflow-hidden",
            button {
                class: "w-full px-6 py-4 hover:bg-zinc-900/50 transition-colors text-left",
                onclick: move |_| expanded.toggle(),
                div { class: "flex items-center justify-between",
                    div { class: "flex-1",
                        div { class: "font-semibold mb-1", "{contact.nickname}" }
                        div { class: "text-sm text-zinc-500", "{contact.username}" }
                    }
                    div { class: "flex items-center gap-4",
                        span { class: "text-sm text-zinc-500", "{contact.devices.len()} devices" }
                        svg {
                            class: if expanded() {
                                "w-5 h-5 text-zinc-500 transform rotate-90 transition-transform"
                            } else {
                                "w-5 h-5 text-zinc-500 transition-transform"
                            },
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
            div { class: "space-y-3 pt-4",
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

    let truncated_id = truncate_endpoint_id(&device.iroh_endpoint_id);

    rsx! {
        div { class: "py-2",
            div { class: "flex items-center justify-between mb-2",
                span { class: "text-sm font-medium text-zinc-300", "{device.name}" }
                span { class: "text-xs font-mono text-zinc-500", "{truncated_id}" }
            }

            if let Some(status) = last_outbound_success() {
                div { class: "text-xs text-zinc-400", "Last outbound: {status.files_transferred} files" }
            }
            if let Some(status) = last_inbound_success() {
                div { class: "text-xs text-zinc-400", "Last inbound: {status.files_transferred} files" }
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
