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

use crate::context::VaultContext;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Main)]
    #[route("/notes")]
    NoteBrowser{},
    
    #[route("/notes/:file_path")]
    NoteView { file_path: String },
    
    #[route("/contacts")]
    ContactBrowser {},
    
    #[route("/contacts/:nickname")]
    ContactView { nickname: String },
    
    #[route("/")]
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
        div { class: "flex flex-col h-screen w-screen bg-gray-900 text-gray-100",
            div { class: "flex flex-row w-full items-center justify-evenly h-12 bg-gray-800 mb-4 text-sm",
                Link {
                    to: Route::NoteBrowser{},
                    "Notes"
                }
                Link {
                    to: Route::ContactBrowser{},
                    "Contacts"
                }
                Link {
                    to: Route::Profile{},
                    "Profile"
                }
            },

            div { class:"flex flex-1 overflow-auto justify-center",
                Outlet::<Route> {}
            }
        }
    }
}
