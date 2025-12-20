use crate::ui::app::Route;
use dioxus::prelude::*;

const NAVBAR_CSS: Asset = asset!("/assets/styling/navbar.css");

#[component]
pub fn Navbar() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: NAVBAR_CSS }

        div {
            id: "navbar",
            Link {
                to: Route::Vault {},
                "Vault"
            }
            Link {
                to: Route::Profile {},
                "Profile"
            }
            Link {
                to: Route::Editor {},
                "Editor"
            }
            Link {
                to: Route::Contacts {},
                "Contacts"
            }
        }

        // The `Outlet` component is used to render the next component inside the layout. In this case, it will render either
        // the [`Home`] or [`Blog`] component depending on the current route.
        Outlet::<Route> {}
    }
}
