use dioxus::prelude::*;

#[component]
pub fn Header(on_menu_click: EventHandler, children: Element) -> Element {
    rsx! {
        header {
            class: "sticky top-0 z-10 border-b border-zinc-800 bg-zinc-900/95 backdrop-blur-sm",
            div {
                class: "flex items-center justify-between px-4 py-3",

                button {
                    class: "p-2 -ml-2 hover:bg-zinc-800 rounded-lg transition-colors",
                    onclick: move |_| on_menu_click.call(()),
                    aria_label: "Menu",
                    svg {
                        class: "w-5 h-5",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            d: "M4 6h16M4 12h16M4 18h16",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                        }
                    }
                }

                {children}
            }
        }
    }
}
