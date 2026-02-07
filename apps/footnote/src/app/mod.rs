pub mod body;
pub mod header;
pub mod modal;
pub mod route;

use dioxus::prelude::*;
use std::env;
use std::path::PathBuf;

use footnote_core::model::vault::{Vault, VaultState};
use footnote_core::util::filesystem::ensure_vault_at_path;
use footnote_core::util::manifest::create_manifest_local;

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
    use_context_provider(|| ImportContactModalVisible(Signal::new(false)));
    use_context_provider(|| ImportedContactString(Signal::new(String::new())));
    use_context_provider(|| PairWithListeningDeviceModalVisible(Signal::new(false)));
    use_context_provider(|| ListeningDeviceUrl(Signal::new(String::new())));
    use_context_provider(|| ListenForPairModalVisible(Signal::new(false)));
    use_context_provider(|| ShareMyContactModalVisible(Signal::new(false)));

    #[cfg(any(target_os = "android", target_os = "ios"))]
    use_hook(|| {
        let Some(mut rx) = take_file_receiver() else {
            tracing::warn!("File receiver already taken!");
            return;
        };

        #[cfg(target_os = "ios")]
        platform::inject_open_url_handler();

        spawn(async move {
            // Handle launch intent (Android)
            #[cfg(target_os = "android")]
            {
                if let Ok(Some(data)) = handle_incoming_share() {
                    consume_context::<ImportedContactString>().set(data);
                    consume_context::<ImportContactModalVisible>().set(true);
                }
            }

            while let Some(incoming_uri) = rx.recv().await {
                tracing::info!("Received data: {}", incoming_uri);

                if incoming_uri.starts_with("footnote+pair://") {
                    tracing::info!("handle join request: {}", incoming_uri);
                    consume_context::<ListeningDeviceUrl>().set(incoming_uri);
                    consume_context::<PairWithListeningDeviceModalVisible>().set(true);
                } else {
                    #[cfg(target_os = "android")]
                    let content = read_uri_from_string(incoming_uri);

                    #[cfg(target_os = "ios")]
                    let content = std::fs::read_to_string(&incoming_uri).ok();

                    if let Some(data) = content {
                        consume_context::<ImportedContactString>().set(data);
                        consume_context::<ImportContactModalVisible>().set(true);
                    }
                }
            }
        });
    });

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
    use_context_provider(|| AppContext {
        vault: Signal::new(vault.clone()),
        vault_state: Signal::new(vault.state_read().unwrap_or(VaultState::Uninitialized)),
        devices: Signal::new(vault.device_read().expect("could not load devices")),
        contacts: Signal::new(vault.contact_read().expect("could not load contacts")),
        manifest: Signal::new(
            create_manifest_local(&vault.base_path()).expect("could not load local list of files"),
        ),
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
fn Main() -> Element {
    let import_contact_modal_visible = use_context::<ImportContactModalVisible>();
    let share_my_contact_modal_visible = use_context::<ShareMyContactModalVisible>();
    let pair_with_listening_device_modal_visible =
        use_context::<PairWithListeningDeviceModalVisible>();
    let listen_for_pair_modal_visible = use_context::<ListenForPairModalVisible>();

    rsx! {
        Header{}

        div {
            class: "flex flex-col flex-1 min-h-screen bg-zinc-950 text-zinc-100 font-sans antialiased pt-safe pb-safe",
            main {
                class: "flex-1 flex flex-col overflow-hidden",
                Outlet::<Route> {}
            }
        }


        if share_my_contact_modal_visible.0() {
            ShareMyContactModal {}
        }

        if import_contact_modal_visible.0() {
            ImportContactModal {}
        }

        if pair_with_listening_device_modal_visible.0() {
            PairWithListeningDeviceModal {}
        }

        if listen_for_pair_modal_visible.0() {
            ListenForPairModal {}
        }

    }
}
