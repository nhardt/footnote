use crate::ui::Screen;
use dioxus::prelude::*;

#[component]
pub fn NavMenuItem(
    screen: Screen,
    current_screen: Signal<Screen>,
    menu_open: Signal<bool>,
    label: String,
    children: Element,
) -> Element {
    rsx! {
        button {
            onclick: move |_| {
                current_screen.set(screen);
                menu_open.set(false);
            },
            class: if current_screen() == screen {
                "flex items-center gap-x-3 rounded-md bg-zinc-800 p-3 text-sm font-semibold text-zinc-100"
            } else {
                "flex items-center gap-x-3 rounded-md p-3 text-sm font-semibold text-zinc-200 hover:bg-zinc-800"
            },
            {children}
            "{label}"
        }
    }
}
