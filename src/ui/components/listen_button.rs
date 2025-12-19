use crate::ui::context::VaultContext;
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
enum ListenStatus {
    Idle,
    Listening { endpoint_id: String },
    Received { from: String, endpoint_id: String },
    Error(String),
}

#[component]
pub fn ListenButton() -> Element {
    let mut listen_status = use_signal(|| ListenStatus::Idle);
    let mut cancel_token = use_signal(|| None::<tokio_util::sync::CancellationToken>);
    let vault_ctx = use_context::<VaultContext>();

    rsx! {
        div { class: "mb-8",
            h2 { class: "text-xl font-bold text-zinc-100 mb-4", "Receive Sync" }
            div { class: "bg-zinc-800 border border-zinc-700 rounded-md p-4",
                div { class: "flex items-center justify-between",
                    div { class: "flex-1",
                        div { class: "font-semibold text-zinc-100", "Accept sync from other devices" }
                        div { class: "text-sm text-zinc-300 mt-1",
                            match listen_status() {
                                ListenStatus::Idle => "Not listening".to_string(),
                                ListenStatus::Listening { ref endpoint_id } => {
                                    format!("Listening on: {}...", &endpoint_id[..16.min(endpoint_id.len())])
                                }
                                ListenStatus::Received { ref from, .. } => {
                                    format!("Recently received sync from: {}", from)
                                }
                                ListenStatus::Error(ref e) => format!("Error: {}", e),
                            }
                        }
                    }
                    if matches!(listen_status(), ListenStatus::Listening { .. } | ListenStatus::Received { .. }) {
                        button {
                            class: "px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-700",
                            onclick: move |_| {
                                // Stop listening
                                if let Some(token) = cancel_token() {
                                    token.cancel();
                                    cancel_token.set(None);
                                    listen_status.set(ListenStatus::Idle);
                                }
                            },
                            "Stop Listening"
                        }
                    } else {
                        button {
                            class: "px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 disabled:bg-zinc-700",
                            disabled: matches!(listen_status(), ListenStatus::Error(_)),
                            onclick: move |_| {
                                let mut listen_status = listen_status.clone();
                                let mut cancel_token = cancel_token.clone();
                                let vault_ctx = vault_ctx.clone();

                                spawn(async move {
                                    let vault_path = match vault_ctx.get_vault() {
                                        Some(path) => path,
                                        None => {
                                            listen_status.set(ListenStatus::Error("No vault path available".to_string()));
                                            return;
                                        }
                                    };

                                    match crate::core::mirror::listen_background(&vault_path).await {
                                        Ok((mut rx, token)) => {
                                            cancel_token.set(Some(token));

                                            // Consume events from the channel
                                            while let Some(event) = rx.recv().await {
                                                match event {
                                                    crate::core::mirror::ListenEvent::Started { endpoint_id } => {
                                                        listen_status.set(ListenStatus::Listening { endpoint_id: endpoint_id.clone() });
                                                    }
                                                    crate::core::mirror::ListenEvent::Received { from } => {
                                                        // Keep the endpoint_id when showing received status
                                                        if let ListenStatus::Listening { endpoint_id } = listen_status() {
                                                            listen_status.set(ListenStatus::Received {
                                                                from: from.clone(),
                                                                endpoint_id: endpoint_id.clone()
                                                            });

                                                            // Reset to listening after 3 seconds
                                                            let mut listen_status = listen_status.clone();
                                                            let endpoint_id_copy = endpoint_id.clone();
                                                            spawn(async move {
                                                                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                                                                if matches!(listen_status(), ListenStatus::Received { .. }) {
                                                                    listen_status.set(ListenStatus::Listening { endpoint_id: endpoint_id_copy });
                                                                }
                                                            });
                                                        }
                                                    }
                                                    crate::core::mirror::ListenEvent::Stopped => {
                                                        listen_status.set(ListenStatus::Idle);
                                                        cancel_token.set(None);
                                                        break;
                                                    }
                                                    crate::core::mirror::ListenEvent::Error(err) => {
                                                        listen_status.set(ListenStatus::Error(err));
                                                        cancel_token.set(None);
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            listen_status.set(ListenStatus::Error(e.to_string()));
                                        }
                                    }
                                });
                            },
                            "Start Listening"
                        }
                    }
                }

                // Show recent sync info
                if let ListenStatus::Received { ref from, .. } = listen_status() {
                    div { class: "mt-2 p-2 bg-zinc-800 border border-green-600 rounded text-sm text-green-400",
                        "Received sync from {from}"
                    }
                }

                if let ListenStatus::Error(ref e) = listen_status() {
                    div { class: "mt-2 p-2 bg-zinc-800 border border-red-600 rounded text-sm text-red-400",
                        "Error: {e}"
                    }
                }
            }
        }
    }
}
