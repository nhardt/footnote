use dioxus::prelude::*;

#[component]
pub fn Contact(petname: String, username: String, device_count: usize) -> Element {
    rsx! {
        div {
            class: "bg-zinc-800 border border-zinc-700 rounded-md p-4 hover:border-zinc-600",
            div { class: "font-semibold text-zinc-100 text-lg", "{petname}" }
            div { class: "text-sm text-zinc-300 mt-1", "{username}" }
            div { class: "text-sm text-zinc-400 mt-2",
                "Devices: {device_count}"
            }
        }
    }
}
