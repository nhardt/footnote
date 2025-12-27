use dioxus::prelude::*;

mod components;
mod context;
mod model;
mod platform;
mod util;
mod views;

use views::contact_browser::ContactBrowser;
use views::contact_view::ContactView;
use views::note_browser::NoteBrowser;
use views::note_view::NoteView;
use views::profile::Profile;

use crate::context::VaultContext;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
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
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    use_context_provider(|| VaultContext::new());
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        div { class: "app-container",
            div { style: "flex-shrink: 0; height: 50px; border-bottom: 1px solid var(--bg-color-tertiary);",
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
            }


            div { style:"display: flex; flex-direction: column; height: 100vh;",
                Router::<Route> {}
            }
        }
        Router::<Route> {}
    }
}
