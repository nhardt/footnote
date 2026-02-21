use dioxus::prelude::*;
use footnote_core::model::device::Device;

use crate::context::AppContext;
use crate::context::MenuContext;
use crate::sync_status_context::SyncStatusContext;

#[component]
pub fn SyncActivity() -> Element {
    let app_context = use_context::<AppContext>();
    let sync_context = use_context::<SyncStatusContext>();

    let user = app_context.user.read().clone();
    let contacts = app_context.contacts.read().clone();
    let my_devices = app_context.devices.read().clone();

    let mut contact_filter: Signal<Option<String>> = use_signal(|| None);
    let mut device_filter: Signal<Option<String>> = use_signal(|| None);

    let recent_files = use_memo(move || {
        let contacts = app_context.contacts.read().clone();
        let my_devices = app_context.devices.read().clone();

        let devices_for_files: Vec<Device> = match contact_filter.read().as_deref() {
            None | Some("__all__") => {
                let mut all = my_devices.clone();
                for c in &contacts {
                    all.extend(c.devices.clone());
                }
                all
            }
            Some(_) => match device_filter.read().as_ref() {
                Some(endpoint_id) => {
                    let available: Vec<Device> = match contact_filter.read().as_deref() {
                        Some("__me__") => my_devices.clone(),
                        Some(key) => contacts
                            .iter()
                            .find(|c| c.id_public_key == key)
                            .map(|c| c.devices.clone())
                            .unwrap_or_default(),
                        _ => vec![],
                    };
                    available
                        .into_iter()
                        .filter(|d| &d.iroh_endpoint_id == endpoint_id)
                        .collect()
                }
                None => match contact_filter.read().as_deref() {
                    Some("__me__") => my_devices.clone(),
                    Some(key) => contacts
                        .iter()
                        .find(|c| c.id_public_key == key)
                        .map(|c| c.devices.clone())
                        .unwrap_or_default(),
                    _ => vec![],
                },
            },
        };

        sync_context.recent_files_for_devices(&devices_for_files)
    });

    // Build the contact list: Me first, then contacts
    let contact_options: Vec<(String, String)> = {
        let mut opts = vec![("__all__".to_string(), "All People".to_string())];
        if let Some(ref u) = user {
            opts.push((
                "__me__".to_string(),
                format!("{} (me!)", u.username.clone()),
            ));
        }
        for c in &contacts {
            let label = if c.nickname.is_empty() {
                c.username.clone()
            } else {
                c.nickname.clone()
            };
            opts.push((c.id_public_key.clone(), label));
        }
        opts
    };

    // Devices for the selected contact
    let available_devices: Vec<Device> = match contact_filter.read().as_deref() {
        None | Some("__all__") => vec![],
        Some("__me__") => my_devices.clone(),
        Some(key) => contacts
            .iter()
            .find(|c| c.id_public_key == key)
            .map(|c| c.devices.clone())
            .unwrap_or_default(),
    };

    // When contact changes, reset device filter
    let mut on_contact_change = move |key: String| {
        contact_filter.set(if key == "__all__" { None } else { Some(key) });
        device_filter.set(None);
    };

    // Devices to pull files from
    let devices_for_files: Vec<Device> = match contact_filter.read().as_deref() {
        None | Some("__all__") => {
            let mut all = my_devices.clone();
            for c in &contacts {
                all.extend(c.devices.clone());
            }
            all
        }
        Some(_) => match device_filter.read().as_ref() {
            Some(endpoint_id) => available_devices
                .iter()
                .filter(|d| &d.iroh_endpoint_id == endpoint_id)
                .cloned()
                .collect(),
            None => available_devices.clone(),
        },
    };

    rsx! {
        div { class: "border border-zinc-800 rounded-lg bg-zinc-900/30",
            div { class: "px-6 py-3 border-b border-zinc-800 flex items-center gap-3",
                span { class: "text-sm font-semibold font-mono text-zinc-400 mr-auto",
                    "Recent Incoming"
                }
                select {
                    class: "bg-zinc-800 border border-zinc-700 rounded-md text-sm text-zinc-300 px-2 py-1 focus:outline-none focus:border-zinc-500 appearance-none",
                    onchange: move |e| on_contact_change(e.value()),
                    for (key, label) in &contact_options {
                        option {
                            value: "{key}",
                            "{label}"
                        }
                    }
                }
                select {
                    class: "bg-zinc-800 border border-zinc-700 rounded-md text-sm text-zinc-300 px-2 py-1 focus:outline-none focus:border-zinc-500 appearance-none",
                    disabled: available_devices.is_empty(),
                    onchange: move |e| {
                        let val = e.value();
                        device_filter.set(if val == "__all__" { None } else { Some(val) });
                    },
                    option { value: "__all__", "All Devices" }
                    for device in &available_devices {
                        option {
                            value: "{device.iroh_endpoint_id}",
                            "{device.name}"
                        }
                    }
                }
            }

            if recent_files.read().is_empty() {
                div {
                    class: "px-6 py-8 text-sm text-zinc-500 text-center",
                    "No recent incoming files"
                }
            } else {
                div {
                    class: "divide-y divide-zinc-800",

                    for file in recent_files.read().iter() {
                        div {
                            class: "px-6 py-3 flex items-center justify-between hover:bg-zinc-900/50 transition-colors cursor-pointer",
                            onclick: {
                                let nav_path = file.filename.to_string().clone();
                                move |_| consume_context::<MenuContext>().go_note(&nav_path)
                            },
                            span {
                                class: "text-sm text-zinc-300 font-mono truncate",
                                "{file.filename}"
                            }
                            span {
                                class: "text-xs text-zinc-500 ml-4 shrink-0",
                                "{file.timestamp.relative_time_string()}"
                            }
                        }
                    }
                }
            }
        }
    }
}
