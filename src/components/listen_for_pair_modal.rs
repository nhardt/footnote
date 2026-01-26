use crate::{
    context::AppContext,
    service::join_service::{JoinEvent, JoinService},
};
use dioxus::prelude::*;
use qrcode_generator::QrCodeEcc;
use tokio_util::sync::CancellationToken;

#[derive(Clone, Copy, PartialEq)]
pub struct ListenForPairModalVisible(pub Signal<bool>);
impl ListenForPairModalVisible {
    pub fn set(&mut self, value: bool) {
        self.0.set(value);
    }
}

#[component]
pub fn ListenForPairModal() -> Element {
    let mut is_listening = use_signal(|| false);
    let mut join_url = use_signal(|| String::new());
    let cancel_token = use_signal(|| CancellationToken::new());
    let mut err_message = use_signal(|| String::new());

    let app_context = use_context::<AppContext>();
    let mut mut_app_context = app_context.clone();

    let img_data = use_memo(move || {
        let svg_data =
            qrcode_generator::to_svg_to_string(join_url(), QrCodeEcc::Medium, 300, None::<&str>)
                .unwrap();
        let safe_svg_data = svg_data.replace(' ', "%20").replace('#', "%23");
        format!("data:image/svg+xml;utf8,{}", safe_svg_data)
    });

    let start_listening = move |_| {
        is_listening.set(true);
        err_message.set(String::new());

        let vault = app_context.vault.read().clone();
        spawn(async move {
            match JoinService::listen(&vault, cancel_token()).await {
                Ok(mut rx) => {
                    while let Some(event) = rx.recv().await {
                        match event {
                            JoinEvent::Listening { join_url: url } => {
                                tracing::info!("received join url for disply {}", join_url);
                                join_url.set(url)
                            }
                            JoinEvent::Success => {
                                tracing::info!("join success");
                                if let Err(e) = mut_app_context.reload() {
                                    tracing::warn!("failed to reload app: {}", e);
                                } else {
                                    tracing::info!("reloaded app");
                                }
                                return;
                            }
                            JoinEvent::Error(e) => err_message.set(format!("{}", e)),
                        }
                    }
                }
                Err(e) => err_message.set(format!("{}", e)),
            }
        });
    };

    let cancel_listening = move |_| {
        cancel_token().cancel();
        consume_context::<ListenForPairModalVisible>().set(false);
    };

    use_drop(move || {
        if is_listening() {
            cancel_token().cancel();
        }
    });

    rsx! {
        div {
            class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50",
            onclick: move |_| consume_context::<ListenForPairModalVisible>().set(false),

            div {
                class: "bg-zinc-900 border border-zinc-800 rounded-lg shadow-2xl max-w-md w-full",
                onclick: move |evt| evt.stop_propagation(),

                div { class: "p-6 border-b border-zinc-800",

                    if !is_listening() {
                        div {
                            p { class: "text-sm text-zinc-500 mb-4",
                                "Connect this device to your primary device"
                            }
                            div { class: "flex gap-3",
                                button {
                                    class: "px-6 py-3 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-lg font-medium transition-all",
                                    onclick: move |_| consume_context::<ListenForPairModalVisible>().set(false),
                                    "Cancel"
                                }
                                button {
                                    class: "px-6 py-3 bg-zinc-100 hover:bg-white text-zinc-900 rounded-lg font-medium transition-all",
                                    onclick: start_listening,
                                    "Listen for Primary Device"
                                }
                            }
                        }
                    } else {
                        div {
                            p { class: "text-sm text-zinc-500 mb-6",
                                "Scan this QR code on your primary device to complete pairing"
                            }

                            div { class: "flex flex-col items-center mb-6",
                                div { class: "bg-white p-4 rounded-lg mb-4",
                                    div { class: "w-48 h-48 bg-zinc-200 flex items-center justify-center text-zinc-500 text-xs",
                                        img {
                                            width: 300,
                                            height: 300,
                                            class: "transition-opacity duration-300",
                                            style: "image-rendering: pixelated;",
                                            src: "{img_data}"
                                        }
                                    }
                                }

                                div { class: "w-full",
                                    label { class: "block text-xs font-medium text-zinc-400 mb-2",
                                        "Join URL"
                                    }
                                    div { class: "bg-zinc-950 border border-zinc-800 rounded-lg p-3",
                                        p { class: "select-all break-all font-mono text-xs text-zinc-400",
                                            "{join_url}"
                                        }
                                    }
                                }
                            }

                            if !err_message().is_empty() {
                                div { class: "mb-4 p-3 bg-red-900/20 border border-red-800 rounded-lg text-sm text-red-400",
                                    "{err_message}"
                                }
                            }

                            button {
                                class: "px-4 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all",
                                onclick: cancel_listening,
                                "Cancel"
                            }
                        }
                    }
                }
            }
        }
    }
}
