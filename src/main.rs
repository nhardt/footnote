use std::path::PathBuf;

use dioxus::prelude::*;

mod components;
mod context;
mod model;
mod platform;
mod service;
mod util;
mod views;

use crate::model::vault::Vault;
use tracing::Level;
use util::filesystem::ensure_default_vault;

use views::contact_browser::ContactBrowser;
use views::contact_view::ContactView;
use views::note_browser::NoteBrowser;
use views::note_view::NoteView;
use views::profile::Profile;

use crate::components::file_service_toggle::FileServiceToggle;
use crate::context::VaultContext;

#[derive(Debug, Clone, Routable, PartialEq)]
enum Route {
    #[layout(Main)]
    #[route("/")]
    NoteBrowser {},

    #[route("/notes/:file_path")]
    NoteView { file_path: String },

    #[route("/contacts")]
    ContactBrowser {},

    #[route("/contacts/:nickname")]
    ContactView { nickname: String },

    #[route("/profile")]
    Profile {},
}
const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::logger::init(Level::TRACE).expect("failed to init logger");
    tracing::trace!("trace");
    tracing::debug!("debug");
    tracing::info!("info");
    tracing::warn!("warn");
    tracing::error!("error");
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let vault_path = ensure_default_vault()?;
    let vault = Vault::new(&vault_path)?;
    use_context_provider(|| VaultContext::new(vault));

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
    rsx! {
        div { class: "flex flex-col flex-1 h-screen bg-zinc-950 text-zinc-100 font-sans antialiased",
            nav { class: "border-b border-zinc-800 bg-zinc-900/50 backdrop-blur-sm",
                div { class: "px-6 py-3",
                    div { class: "flex items-center gap-8",
                        Link {
                            class: if matches!(route, Route::NoteBrowser {} | Route::NoteView { .. }) {
                                "px-4 py-2 text-sm font-medium text-zinc-100 border-b-2 border-zinc-100"
                            } else {
                                "px-4 py-2 text-sm font-medium text-zinc-400 hover:text-zinc-100 transition-colors"
                            },
                            to: Route::NoteBrowser{}, "Notes" }
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
                            FileServiceToggle{}
                        }
                    }
                }
            }

            main {
                // todo: let child component control render rather than customizing here
                class: if matches!(route, Route::NoteView { .. }) {
                    "flex-1 flex flex-col"
                } else {
                    "max-w-3xl mx-auto px-6 py-12"
                },
                Outlet::<Route> {}
            }
        }
    }
}
