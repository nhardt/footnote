use dioxus::prelude::*;

use crate::ui::views::{Contacts, Editor, Navbar, Profile, Vault};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Navbar())]
        #[route("/")]
        Vault {},
        
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

#[component]
pub fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        Router::<Route> {}
    }
}
