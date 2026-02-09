use dioxus::prelude::*;

use crate::context::{AppContext, MenuContext};

#[cfg(any(target_os = "android", target_os = "ios"))]
use crate::platform::SHARE_SHEET_SUPPORTED;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
const SHARE_SHEET_SUPPORTED: bool = false;

#[component]
pub fn ShareMyContactModal() -> Element {
    let app_context = use_context::<AppContext>();
    let mut error_message = use_signal(|| None::<String>);

    let user_record_json = use_signal(move || match app_context.vault.read().user_read() {
        Ok(Some(user)) => user
            .to_json_pretty()
            .unwrap_or_else(|e| format!("Failed to serialize: {}", e)),
        Ok(None) => "User record not found".to_string(),
        Err(e) => format!("Contact record unable to load: {}", e),
    });

    #[allow(unused)]
    let handle_share = move |_| {
        #[cfg(any(target_os = "android", target_os = "ios"))]
        use crate::platform;
        error_message.set(None);

        let Some((username, timestamp, Some(json_content))) = app_context
            .vault
            .read()
            .user_read()
            .ok()
            .flatten()
            .map(|u| (u.username.clone(), u.updated_at, u.to_json_pretty().ok()))
        else {
            error_message.set(Some("failed to get user record".into()));
            return;
        };

        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!("{}.{}.fncontact", username, timestamp));

        match std::fs::write(&file_path, json_content) {
            Ok(_) =>
            {
                #[cfg(any(target_os = "android", target_os = "ios"))]
                if let Err(e) = platform::share_contact_file(&file_path) {
                    error_message.set(Some(format!("Share failed: {}", e)));
                }
            }
            Err(e) => {
                error_message.set(Some(format!("Failed to create file: {}", e)));
            }
        }
    };

    rsx! {
        div {
            class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50",
            div {
                class: "bg-zinc-900 border border-zinc-800 rounded-lg shadow-2xl w-full max-w-2xl h-[80vh] flex flex-col",
                onclick: move |evt| evt.stop_propagation(),
                div { class: "p-6 border-b border-zinc-800",
                    h3 { class: "text-lg text-zinc-300 font-semibold font-mono", "Export Contact Record" }
                    p { class: "text-sm text-zinc-500 mt-1",
                        "Share this with your trusted contacts"
                    }
                }
                div { class: "p-6 flex-1 min-h-0 flex flex-col",
                    textarea {
                        class: "flex-1 w-full px-4 py-3 bg-zinc-950 border border-zinc-800 rounded-lg text-xs font-mono text-zinc-300 resize-none focus:border-zinc-600 focus:ring-1 focus:ring-zinc-600 mb-4",
                        "{user_record_json}"
                    }

                    if let Some(error) = error_message() {
                        div { class: "mb-4 p-3 bg-red-900/20 border border-red-800 rounded-lg text-sm text-red-400",
                            "{error}"
                        }
                    }

                    {
                        if SHARE_SHEET_SUPPORTED {
                            rsx! {
                                div { class: "flex gap-3",
                                    button {
                                        class: "flex-1 px-4 py-2 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 rounded-md text-sm font-medium transition-all",
                                        onclick: handle_share,
                                        "Share"
                                    }
                                    button {
                                        class: "flex-1 px-4 py-2 bg-zinc-300 hover:bg-white text-zinc-900 rounded-md text-sm font-medium transition-all",
                                        onclick: move |_| consume_context::<MenuContext>().close_all(),
                                        "Done"
                                    }
                                }
                            }
                        } else {
                            rsx! {
                                button {
                                    class: "w-full px-4 py-2 bg-zinc-300 hover:bg-white text-zinc-900 rounded-md text-sm font-medium transition-all",
                                    onclick: move |_| consume_context::<MenuContext>().close_all(),
                                    "Done"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
