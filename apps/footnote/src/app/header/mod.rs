use dioxus::prelude::*;

pub mod menu;

#[component]
pub fn Header(children: Element) -> Element {
    rsx! {
        header {
            class: "sticky top-0 z-10 border-b border-zinc-800 bg-zinc-900/95 backdrop-blur-sm",
            div {
                class: "flex items-center justify-between px-4 py-3",

                HeaderMenu {}
            }
        }
    }
}
