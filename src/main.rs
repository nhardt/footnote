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
use crate::components::import_contact_modal::{
    ImportContactModal, ImportContactModalVisible, ImportedContactString,
};
use crate::components::sync_service_toggle::SyncServiceToggle;
use crate::context::AppContext;
use crate::model::vault::{Vault, VaultState};
use crate::util::manifest::create_manifest_local;
use tracing::Level;
use util::filesystem::ensure_default_vault;
use views::contact_view::ContactBrowser;
use views::note_view::NoteView;
use views::profile_view::Profile;

#[cfg(target_os = "android")]
use {
    crate::platform::{send_incoming_file, take_file_receiver},
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

    #[route("/notes/:file_path")]
    NoteView { file_path: String },

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
    use_context_provider(|| ImportContactModalVisible(Signal::new(false)));
    use_context_provider(|| ImportedContactString(Signal::new(String::new())));

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
                tracing::info!("Received file: {}", incoming_uri);

                #[cfg(target_os = "android")]
                let content = read_uri_from_string(incoming_uri);

                #[cfg(target_os = "ios")]
                let content = std::fs::read_to_string(&incoming_uri).ok();

                if let Some(data) = content {
                    consume_context::<ImportedContactString>().set(data);
                    consume_context::<ImportContactModalVisible>().set(true);
                }
            }
        });
    });

    let vault_path = ensure_default_vault()?;
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

        Router::<Route> {}
    }
}

#[component]
fn Main() -> Element {
    let route = use_route::<Route>();
    let contact_modal_visible = use_context::<ImportContactModalVisible>();

    rsx! {
        div { class: "flex flex-col flex-1 h-screen bg-zinc-950 text-zinc-100 font-sans antialiased",
            nav { class: "border-b border-zinc-800 bg-zinc-900/50 backdrop-blur-sm",
                div { class: "px-6 py-3",
                    div { class: "flex items-center gap-8",
                        Link {
                            class: if matches!(route, Route::NoteDefault {} | Route::NoteView { .. }) {
                                "px-4 py-2 text-sm font-medium text-zinc-100 border-b-2 border-zinc-100"
                            } else {
                                "px-4 py-2 text-sm font-medium text-zinc-400 hover:text-zinc-100 transition-colors"
                            },
                            to: Route::NoteDefault{}, "Notes" }
                        Link {
                            class: if matches!(route, Route::Profile{}) {
                                "px-4 py-2 text-sm font-medium text-zinc-100 border-b-2 border-zinc-100"
                            } else {
                                "px-4 py-2 text-sm font-medium text-zinc-400 hover:text-zinc-100 transition-colors"
                            },
                            to: Route::Profile{}, "Profile" }
                        Link {
                            class: if matches!(route, Route::ContactBrowser{}) {
                                "px-4 py-2 text-sm font-medium text-zinc-100 border-b-2 border-zinc-100"
                            } else {
                                "px-4 py-2 text-sm font-medium text-zinc-400 hover:text-zinc-100 transition-colors"
                            },
                            to: Route::ContactBrowser {},
                            "Contact"}
                        div { class: "ml-auto flex items-center gap-2",
                            div { class: "h-2 w-2 rounded-full bg-zinc-500" }
                            SyncServiceToggle{}
                        }
                    }
                }
            }

            match route {
                Route::NoteView { file_path } => rsx! {
                    main {
                        class: "flex-1 flex flex-col",
                        key: "{file_path}",  // This forces remount on file_path change
                        Outlet::<Route> {}
                    }
                },
                _ => rsx! {
                    main {
                        class:"max-w-3xl mx-auto px-6 py-12",
                        Outlet::<Route> {}
                    }

                }
            }
        }

        if contact_modal_visible.0() {
            ImportContactModal {}
        }
    }
}

#[component]
fn NoteDefault() -> Element {
    let nav = navigator();
    let app_context = use_context::<AppContext>();
    use_effect(move || {
        nav.push(Route::NoteView {
            file_path: urlencoding::encode(
                &app_context
                    .vault
                    .read()
                    .base_path()
                    .join("home.md")
                    .to_string_lossy()
                    .to_string(),
            )
            .to_string(),
        });
    });
    rsx! {
        div { class: "flex items-center justify-center h-screen",
            "Loading..."
        }
    }
}
