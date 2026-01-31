use dioxus::prelude::*;
use std::path::PathBuf;
mod components;
mod context;
mod elements;
mod model;
mod platform;
mod service;
mod util;
mod views;
use crate::components::app_menu::AppMenuVisible;
use crate::components::import_contact_modal::{
    ImportContactModal, ImportContactModalVisible, ImportedContactString,
};
use crate::components::listen_for_pair_modal::{
    self, ListenForPairModal, ListenForPairModalVisible,
};
use crate::components::pair_with_listening_device_modal::{
    ListeningDeviceUrl, PairWithListeningDeviceModal, PairWithListeningDeviceModalVisible,
};
use crate::components::share_my_contact_modal::{ShareMyContactModal, ShareMyContactModalVisible};
use crate::components::sync_service_toggle::SyncServiceToggle;
use crate::context::AppContext;
use crate::model::vault::{Vault, VaultState};
use crate::util::manifest::create_manifest_local;
use tracing::Level;
use util::filesystem::hack_ensure_default_vault;
use views::contact_view::ContactBrowser;
use views::note_view::NoteView;
use views::profile_view::Profile;

#[cfg(target_os = "android")]
use {
    crate::platform::{
        handle_incoming_share, read_uri_from_string, send_incoming_file, take_file_receiver,
    },
    std::sync::mpsc::{channel, Receiver, Sender},
    std::sync::Mutex,
    std::sync::OnceLock,
};

#[cfg(target_os = "ios")]
use crate::platform::{send_incoming_file, take_file_receiver};

#[derive(Debug, Clone, Routable, PartialEq)]
enum Route {
    #[layout(Main)]
    #[route("/")]
    NoteDefault {},

    #[route("/notes/:..file_path_segments")]
    NoteView { file_path_segments: Vec<String> },

    #[route("/contacts")]
    ContactBrowser {},

    #[route("/profile")]
    Profile {},
}
const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::logger::init(Level::INFO).expect("failed to init logger");
    tracing::trace!("trace");
    tracing::debug!("debug");
    tracing::info!("info");
    tracing::warn!("warn");
    tracing::error!("error");
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // this no longer seems like a good way to do this :crying_laughing_face:
    use_context_provider(|| AppMenuVisible(Signal::new(false)));

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

    let vault_path = hack_ensure_default_vault()?;
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
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Stylesheet { href: MAIN_CSS }
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

#[component]
fn NoteDefault() -> Element {
    let nav = navigator();
    let app_context = use_context::<AppContext>();
    use_effect(move || {
        nav.push(format!(
            "/notes/{}",
            &app_context
                .vault
                .read()
                .base_path()
                .join("home.md")
                .to_string_lossy()
                .to_string(),
        ));
    });
    rsx! {
        div { class: "flex items-center justify-center h-screen",
            "Loading..."
        }
    }
}
