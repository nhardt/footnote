use dioxus::prelude::*;

#[component]
pub fn App() -> Element {
    rsx! {
        document::Stylesheet { href: asset!("/assets/tailwind.css") }
        div { class: "bg-red-100", "Make a Footnote!" }
    }
}
