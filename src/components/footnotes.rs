use dioxus::prelude::*;
use indexmap::IndexMap;

#[component]
pub fn Footnotes(footnotes: ReadSignal<IndexMap<String, String>>) -> Element {
    rsx! {
        div { class: "overflow-hidden rounded-lg border border-zinc-800 bg-zinc-900/30",
            div { class: "py-3 px-4 border-b border-zinc-800 bg-zinc-900/50",
                h3 { class: "text-sm font-semibold text-zinc-300",
                    "Footnotes"
                }
            }

            if footnotes().len() == 0 {
                div { class: "py-8 px-4 text-center",
                    p { class: "text-sm italic text-zinc-500",
                        "No footnotes found. Use [^name] to add references."
                    }
                }
            }

            for footnote in footnotes() {
                div { class: "py-3 px-4 transition-colors group hover:bg-zinc-800/50",
                    div { class: "flex gap-3 items-center",
                        span { class: "flex-shrink-0 w-20 font-mono text-xs text-zinc-500",
                            "{footnote.0}"
                        }
                        button { class: "flex flex-1 gap-2 items-center text-sm text-left transition-colors text-zinc-300 hover:text-zinc-100",
                            span { "{footnote.1}" }
                        }
                    }
                }
            }
        }
    }
}
