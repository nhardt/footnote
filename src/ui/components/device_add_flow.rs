use crate::ui::context::VaultContext;
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub enum DeviceAddState {
    Idle,
    Listening { join_url: String },
    Connecting,
    ReceivedRequest { device_name: String },
    Verifying,
    Success { device_name: String },
    Error(String),
}

#[component]
pub fn DeviceAddFlow(
    device_add_state: Signal<DeviceAddState>,
    reload_trigger: Signal<i32>,
) -> Element {
    let vault_ctx = use_context::<VaultContext>();

    rsx! {
        div {
            // Add Device button
            div { class: "flex items-center justify-between mb-4",
                h2 { class: "text-xl font-bold text-zinc-100", "My Devices" }
                if matches!(device_add_state(), DeviceAddState::Idle) {
                    button {
                        class: "px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-700",
                        onclick: move |_| {
                            let mut device_add_state = device_add_state.clone();
                            let mut reload_trigger = reload_trigger.clone();
                            let vault_ctx = vault_ctx.clone();

                            spawn(async move {
                                // Get vault path from context
                                let vault_path = match vault_ctx.get_vault() {
                                    Some(path) => path,
                                    None => {
                                        device_add_state.set(DeviceAddState::Error(
                                            "No vault path available".to_string()
                                        ));
                                        return;
                                    }
                                };

                                match crate::core::device::create_primary(&vault_path).await {
                                    Ok(mut rx) => {
                                        // Consume events from the channel
                                        while let Some(event) = rx.recv().await {
                                            match event {
                                                crate::core::device::DeviceAuthEvent::Listening { join_url } => {
                                                    device_add_state.set(DeviceAddState::Listening { join_url });
                                                }
                                                crate::core::device::DeviceAuthEvent::Connecting => {
                                                    device_add_state.set(DeviceAddState::Connecting);
                                                }
                                                crate::core::device::DeviceAuthEvent::ReceivedRequest { device_name } => {
                                                    device_add_state.set(DeviceAddState::ReceivedRequest { device_name });
                                                }
                                                crate::core::device::DeviceAuthEvent::Verifying => {
                                                    device_add_state.set(DeviceAddState::Verifying);
                                                }
                                                crate::core::device::DeviceAuthEvent::Success { device_name } => {
                                                    device_add_state.set(DeviceAddState::Success { device_name });
                                                    // Reload contacts
                                                    reload_trigger.set(reload_trigger() + 1);
                                                    break;
                                                }
                                                crate::core::device::DeviceAuthEvent::Error(err) => {
                                                    device_add_state.set(DeviceAddState::Error(err));
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        device_add_state.set(DeviceAddState::Error(e.to_string()));
                                    }
                                }
                            });
                        },
                        "Add Device"
                    }
                }
            }

            // Device pairing UI
            match device_add_state() {
                DeviceAddState::Listening { ref join_url } => rsx! {
                    div { class: "mt-4 p-4 bg-zinc-800 border border-yellow-600 rounded-md",
                        div { class: "font-semibold text-zinc-100 mb-2", "🔐 Waiting for device..." }
                        div { class: "text-sm text-zinc-300 mb-2", "Copy this URL to your new device:" }
                        div { class: "font-mono text-xs bg-zinc-900 text-zinc-300 p-2 rounded border border-zinc-700 break-all",
                            "{join_url}"
                        }
                        div { class: "text-sm text-zinc-400 mt-2 italic",
                            "Listening for connection..."
                        }
                    }
                },
                DeviceAddState::Connecting => rsx! {
                    div { class: "mt-4 p-4 bg-zinc-800 border border-blue-600 rounded-md",
                        div { class: "font-semibold text-zinc-100", "✓ Device connecting..." }
                    }
                },
                DeviceAddState::ReceivedRequest { ref device_name } => rsx! {
                    div { class: "mt-4 p-4 bg-zinc-800 border border-blue-600 rounded-md",
                        div { class: "font-semibold text-zinc-100", "✓ Received request from: {device_name}" }
                    }
                },
                DeviceAddState::Verifying => rsx! {
                    div { class: "mt-4 p-4 bg-zinc-800 border border-blue-600 rounded-md",
                        div { class: "font-semibold text-zinc-100", "✓ Verifying..." }
                    }
                },
                DeviceAddState::Success { ref device_name } => rsx! {
                    div { class: "mt-4 p-4 bg-zinc-800 border border-green-600 rounded-md",
                        div { class: "font-semibold text-zinc-100", "✓ Device '{device_name}' added successfully!" }
                        button {
                            class: "mt-2 text-sm text-indigo-400 hover:underline",
                            onclick: move |_| device_add_state.set(DeviceAddState::Idle),
                            "Done"
                        }
                    }
                },
                DeviceAddState::Error(ref error) => rsx! {
                    div { class: "mt-4 p-4 bg-zinc-800 border border-red-600 rounded-md",
                        div { class: "font-semibold text-red-400", "✗ Error" }
                        div { class: "text-sm text-zinc-300 mt-1", "{error}" }
                        button {
                            class: "mt-2 text-sm text-indigo-400 hover:underline",
                            onclick: move |_| device_add_state.set(DeviceAddState::Idle),
                            "Try Again"
                        }
                    }
                },
                DeviceAddState::Idle => rsx! {},
            }
        }
    }
}
