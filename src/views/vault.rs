use dioxus::prelude::*;

use crate::components::Hero;

#[component]
pub fn Vault() -> Element {
    rsx! {
       Hero {}
    }
}
