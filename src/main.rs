use dioxus::prelude::*;

mod components;
mod context;
mod platform;
mod views;

use components::Navbar;
use views::{Contacts, Editor, Profile, VaultCreate, VaultHome};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
        #[route("/vault/home")]
        VaultHome {},

        #[route("/vault/create")]
        VaultCreate {},

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
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        div { class: "h-screen flex flex-col bg-zinc-950",
            Router::<Route> {}
        }
    }
}
