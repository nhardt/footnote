use std::path::PathBuf;

use dioxus::prelude::*;

mod components;
mod context;
mod model;
mod platform;
mod service;
mod util;
mod views;

use model::vault::Vault;
use util::filesystem::ensure_default_vault;

use views::contact_browser::ContactBrowser;
use views::contact_view::ContactView;
use views::note_browser::NoteBrowser;
use views::note_view::NoteView;
use views::profile::Profile;

use crate::components::file_service_toggle::FileServiceToggle;
use crate::context::VaultContext;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Main)]
    #[route("/")]
    NoteBrowser{},
    
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

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let vault_path = ensure_default_vault()?;
    use_context_provider(|| VaultContext::new(Some(vault_path)));

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        Router::<Route> {}
    }
}

#[component]
fn Main() -> Element {
    rsx! {
        div { class: "h-screen bg-zinc-950 text-zinc-100 font-sans antialiased",
            nav { class: "border-b border-zinc-800 bg-zinc-900/50 backdrop-blur-sm",
                div { class: "px-6 py-3",
                    div { class: "flex items-center gap-8",
                        Link {
                            class: "px-4 py-2 text-sm font-medium text-zinc-400 hover:text-zinc-100 transition-colors",
                            to: Route::NoteBrowser{}, "Notes" }
                        Link {
                            class: "px-4 py-2 text-sm font-medium text-zinc-100 border-b-2 border-zinc-100",
                            to: Route::Profile{}, "Profile" }
                        Link {
                            class: "px-4 py-2 text-sm font-medium text-zinc-400 hover:text-zinc-100 transition-colors",
                            to: Route::ContactBrowser {},
                            "Contact"}
                        div { class: "ml-auto flex items-center gap-2",
                            div { class: "h-2 w-2 rounded-full bg-zinc-500" }
                            FileServiceToggle{}
                        }
                    }
                }
            }
            main { class: "max-w-3xl mx-auto px-6 py-12",
                div { class: "mb-12",
                    Outlet::<Route> {}
                }
            }
        }
    }
}
