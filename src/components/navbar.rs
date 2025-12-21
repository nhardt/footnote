use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn Navbar() -> Element {
    rsx! {
        div { class: "flex h-flex",
            Link {
                to: Route::VaultHome {},
                "Vault"
            }
            Link {
                to: Route::Editor {},
                "Editor"
            }
            Link {
                to: Route::Profile {},
                "Profile"
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
