use dioxus::prelude::*;

#[component]
pub fn Hero() -> Element {
    rsx! {
        div {
            id: "hero",
            "Welcome back"
        }
    }
}
