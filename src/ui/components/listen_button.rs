use crate::ui::components::icons;
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

    let is_listening = matches!(
        listen_status(),
        ListenStatus::Listening { .. } | ListenStatus::Received { .. }
    );

    rsx! {
        button {
            class: if is_listening {
                "p-2 rounded-md bg-green-600 text-white hover:bg-green-700"
            } else {
                "p-2 rounded-md bg-zinc-700 text-zinc-400 hover:bg-zinc-600 hover:text-zinc-100"
            },
            onclick: move |_| {
                if is_listening {
                    // Stop listening
                    if let Some(token) = cancel_token() {
                        token.cancel();
                        cancel_token.set(None);
                        listen_status.set(ListenStatus::Idle);
                    }
                } else {
                    // Start listening
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
                }
            },
            span { class: "sr-only", if is_listening { "Stop Listening" } else { "Start Listening" } }
            icons::WifiIcon {}
        }
    }
}
