use dioxus::prelude::*;

mod components;
mod context;
mod platform;
mod views;

use components::Navbar;
use views::{Contacts, Editor, Profile, VaultCreate, VaultHome, VaultJoin, VaultOpen};

use crate::context::VaultContext;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
        #[route("/")]
        VaultHome {},

        #[route("/vault-create")]
        VaultCreate {},

        #[route("/vault-open")]
        VaultOpen {},

        #[route("/vault-join")]
        VaultJoin {},

        #[route("/profile")]
        Profile {},

        #[route("/editor")]
        Editor {},

        #[route("/contacts")]
        Contacts {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    use_context_provider(|| VaultContext::new());
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        Router::<Route> {}
    }
}
