pub mod body;
pub mod header;
pub mod modal;
pub mod route;

use dioxus::prelude::*;
use std::env;
use std::path::PathBuf;

use footnote_core::model::vault::Vault;
use footnote_core::util::filesystem::ensure_vault_at_path;

use crate::context::AppContext;
use crate::context::MenuContext;
use crate::header::Header;
use crate::modal::import_contact_modal::ImportContactModal;
use crate::modal::listen_for_pair_modal::ListenForPairModal;
use crate::modal::new_note_modal::NewNoteModal;
use crate::modal::note_browser_modal::NoteBrowserModal;
use crate::modal::pair_with_listening_device_modal::PairWithListeningDeviceModal;
use crate::modal::share_my_contact_modal::ShareMyContactModal;
use crate::route::Route;
use crate::sync_status_context::SyncStatusContext;

#[cfg(target_os = "android")]
use {
    crate::platform::{
        handle_incoming_share, read_uri_from_string, send_incoming_file, take_file_receiver,
    },
    std::sync::Mutex,
    std::sync::OnceLock,
};

#[cfg(target_os = "ios")]
use crate::platform::{send_incoming_file, take_file_receiver};

#[unsafe(no_mangle)]
pub extern "C" fn start_footnote_app() {
    dioxus::launch(App);
}

#[component]
pub fn App() -> Element {
    use_context_provider(MenuContext::new);

    let path_key = "FOOTNOTE_PATH";
    let vault_name_key = "FOOTNOTE_VAULT";

    let vault_path = match env::var(path_key) {
        Ok(ref val) if val.is_empty() => crate::platform::get_app_dir()?,
        Ok(val) => PathBuf::from(val),
        Err(_) => crate::platform::get_app_dir()?,
    };

    let vault_name = match env::var(vault_name_key) {
        Ok(ref val) if val.is_empty() => "footnote.wiki".to_string(),
        Ok(val) => val,
        Err(_) => "footnote.wiki".to_string(),
    };

    let vault_path = ensure_vault_at_path(&vault_path, &vault_name)?;
    let vault = Vault::new(&vault_path)?;
    use_context_provider(|| AppContext::new(vault.clone()));

    use_context_provider(|| SyncStatusContext::new(&vault.clone()));

    #[cfg(any(target_os = "android", target_os = "ios"))]
    use_hook(|| {
        let Some(mut rx) = take_file_receiver() else {
            tracing::warn!("File receiver already taken!");
            return;
        };

        #[cfg(target_os = "ios")]
        crate::platform::inject_open_url_handler();

        spawn(async move {
            // Handle launch intent (Android)
            #[cfg(target_os = "android")]
            {
                if let Ok(Some(data)) = handle_incoming_share() {
                    consume_context::<MenuContext>().set_import_contact_visible(&data);
                }
            }

            while let Some(incoming_uri) = rx.recv().await {
                tracing::info!("Received data: {}", incoming_uri);

                if incoming_uri.starts_with("footnote+pair://") {
                    tracing::info!("handle join request: {}", incoming_uri);
                    consume_context::<MenuContext>().set_pair_with_listener_visible(&incoming_uri);
                } else {
                    #[cfg(target_os = "android")]
                    let content = read_uri_from_string(incoming_uri);

                    #[cfg(target_os = "ios")]
                    let content = std::fs::read_to_string(&incoming_uri).ok();

                    if let Some(data) = content {
                        consume_context::<MenuContext>().set_import_contact_visible(&data);
                    }
                }
            }
        });
    });

    rsx! {
        {
            #[cfg(not(target_os = "ios"))]
            rsx! {
                document::Stylesheet { href: asset!("/assets/tailwind.css") }
                document::Stylesheet { href: asset!("/assets/main.css") }
            }
        }
        {
            #[cfg(target_os = "ios")]
            rsx! {
                document::Link { rel: "stylesheet", href: "/assets/tailwind.css" }
                document::Link { rel: "stylesheet", href: "/assets/main.css" }
            }
        }
        document::Meta {
            name: "viewport",
            content: "width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no, viewport-fit=cover",
        }
        Router::<Route> {}
    }
}

#[component]
pub fn Main() -> Element {
    rsx! {
       div {
            class: "flex flex-col flex-1 min-h-screen bg-zinc-950 text-zinc-100 font-sans antialiased pt-safe pb-safe",
            Header {}
            main {
                class: "flex-1 flex flex-col overflow-hidden",
                Outlet::<Route> {}
            }
        }

        if *consume_context::<MenuContext>().new_note_visible.read() {
            NewNoteModal{}
        }

        if *consume_context::<MenuContext>().note_browser_visible.read() {
            NoteBrowserModal {}
        }

        if *consume_context::<MenuContext>().import_contact_visible.read() {
            ImportContactModal {}
        }

        if *consume_context::<MenuContext>().share_contact_visible.read() {
            ShareMyContactModal {}
        }

        if *consume_context::<MenuContext>().listen_for_pair_visible.read() {
            ListenForPairModal {}
        }

        if *consume_context::<MenuContext>().pair_with_listener_visible.read() {
            PairWithListeningDeviceModal {}
        }
    }
}
