use dioxus::prelude::*;

#[component]
pub fn PrimaryButton(children: Element, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            class: "py-2 px-4 text-sm font-medium rounded-md border transition-all bg-zinc-800 border-zinc-700 hover:bg-zinc-700 hover:border-zinc-600",
            onclick: onclick,
            {children}
        }
    }
}
