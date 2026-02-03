use dioxus::prelude::*;

#[component]
pub fn ConfirmDialog(
    children: Element,
    onconfirm: EventHandler,
    oncancel: EventHandler,
) -> Element {
    rsx! {
        div {
            class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50",
            onclick: move |evt| evt.stop_propagation(),
            div {
                class: "bg-zinc-900 border border-zinc-800 rounded-lg shadow-2xl max-w-sm w-full",
                div { class: "p-6",
                    {children}
                    div { class: "flex gap-3 justify-end",
                        button {
                            class: "px-4 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 rounded-md text-sm font-medium transition-all",
                            onclick: move |_| oncancel.call(()),
                            "Cancel"
                        }
                        button {
                            class: "px-4 py-2 bg-red-600 hover:bg-red-700 rounded-md text-sm font-medium transition-all",
                            onclick: move |_| onconfirm.call(()),
                            "Delete"
                        }
                    }
                }
            }
        }
    }
}
