use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn MenuButton(label: &'static str, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            class: "w-full px-4 py-3 text-left text-sm text-zinc-300 hover:bg-zinc-800 hover:text-zinc-100 rounded-lg transition-colors",
            onclick: onclick,
            "{label}"
        }
    }
}

#[component]
pub fn MenuDivider() -> Element {
    rsx! {
        div {
            class: "my-2 border-t border-zinc-800"
        }
    }
}

#[component]
pub fn AppMenu(visible: bool, on_close: EventHandler, children: Element) -> Element {
    let nav = use_navigator();

    if !visible {
        return rsx! {};
    }

    rsx! {
        div {
            class: "fixed inset-0 bg-black/60 backdrop-blur-sm z-40",
            onclick: move |_| on_close.call(()),
        }

        div {
            class: "fixed inset-y-0 left-0 w-72 bg-zinc-900 border-r border-zinc-800 z-50 shadow-2xl flex flex-col",
            onclick: move |e| e.stop_propagation(),

            div {
                class: "p-4 border-b border-zinc-800",
                h2 {
                    class: "text-lg font-semibold text-zinc-100",
                    "Footnote"
                }
            }

            nav {
                class: "flex-1 overflow-y-auto p-2",
                {children}
            }

            div {
                class: "p-2 border-t border-zinc-800",

                MenuButton {
                    label: "Notes",
                    onclick: move |_| {
                        nav.push(Route::NoteDefault {});
                        on_close.call(());
                    }
                }

                MenuButton {
                    label: "Profile",
                    onclick: move |_| {
                        nav.push(Route::Profile {});
                        on_close.call(());
                    }
                }

                MenuButton {
                    label: "Contacts",
                    onclick: move |_| {
                        nav.push(Route::ContactBrowser {});
                        on_close.call(());
                    }
                }
            }
        }
    }
}
