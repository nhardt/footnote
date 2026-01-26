use crate::{context::AppContext, service::join_service::JoinService};
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub struct IncludeDeviceModalVisible(pub Signal<bool>);

impl IncludeDeviceModalVisible {
    pub fn set(&mut self, value: bool) {
        self.0.set(value);
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct ListeningDeviceUrl(pub Signal<String>);

impl ListeningDeviceUrl {
    pub fn set(&mut self, value: String) {
        self.0.set(value);
    }
}

#[component]
pub fn IncludeDeviceModal() -> Element {
    let mut join_url = use_signal(|| String::new());
    let mut err_message = use_signal(|| String::new());
    let mut is_connecting = use_signal(|| false);
    let mut app_context = use_context::<AppContext>();

    let listening_device_url = use_context::<ListeningDeviceUrl>();
    use_effect(move || {
        let url = listening_device_url.0.read();
        if !url.is_empty() {
            join_url.set(url.clone());
        }
    });

    let connect_to_device = move |_| {
        let url = join_url.read().clone();
        if url.is_empty() {
            err_message.set("Please enter a join URL".to_string());
            return;
        }

        is_connecting.set(true);
        err_message.set(String::new());

        let vault = app_context.vault.read().clone();
        spawn(async move {
            match JoinService::join(&vault, &url).await {
                Ok(_) => {
                    if let Err(e) = app_context.reload() {
                        tracing::warn!("failed to reload app: {}", e);
                        err_message.set(format!("Connected but failed to reload: {}", e));
                    } else {
                        consume_context::<IncludeDeviceModalVisible>().set(false);
                    }
                }
                Err(e) => {
                    err_message.set(format!("Failed to connect: {}", e));
                    is_connecting.set(false);
                }
            }
        });
    };

    rsx! {
        div {
            id: "include-device-modal",
            class: "fixed text-zinc-100 inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50",
            div {
                class: "bg-zinc-900 border border-zinc-800 rounded-lg shadow-2xl max-w-md w-full",
                onclick: move |evt| evt.stop_propagation(),
                div { class: "p-6 border-b border-zinc-500",
                    h3 { class: "text-lg font-semibold font-mono",
                        "Import Device to Group"
                    }
                }
                div { class: "p-6 flex flex-col gap-4",
                    div {
                        label { class: "block text-sm font-medium text-zinc-300 mb-2",
                            "Join URL"
                        }
                        input {
                            class: "w-full px-3 py-2 bg-zinc-950 border border-zinc-700 rounded-md text-sm font-mono focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500",
                            placeholder: "footnote+pair://...",
                            r#type: "text",
                            value: "{join_url}",
                            oninput: move |e| join_url.set(e.value()),
                            disabled: is_connecting()
                        }
                    }
                    if !err_message().is_empty() {
                        div { class: "p-3 bg-red-900/20 border border-red-800 rounded-lg text-sm text-red-400",
                            "{err_message}"
                        }
                    }
                    div { class: "flex gap-3",
                        button {
                            class: "flex-1 px-4 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all",
                            onclick: move |_| consume_context::<IncludeDeviceModalVisible>().set(false),
                            disabled: is_connecting(),
                            "Cancel"
                        }
                        button {
                            class: "flex-1 px-4 py-2 bg-zinc-100 hover:bg-white text-zinc-900 rounded-md text-sm font-medium transition-all disabled:opacity-50 disabled:cursor-not-allowed",
                            onclick: connect_to_device,
                            disabled: is_connecting(),
                            if is_connecting() {
                                "Connecting..."
                            } else {
                                "Connect"
                            }
                        }
                    }
                }
            }
        }
    }
}
