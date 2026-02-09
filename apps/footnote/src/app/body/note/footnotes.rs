use dioxus::prelude::*;

use indexmap::IndexMap;

use crate::body::note::footnote_editor::FootnoteEditor;

#[component]
pub fn Footnotes(
    footnotes: ReadSignal<IndexMap<String, String>>,
    onlink: EventHandler<(String, String)>,
    onlinkclick: EventHandler<String>,
) -> Element {
    let mut editing_footnote = use_signal(|| None::<(String, String)>);

    rsx! {
        div { class: "overflow-hidden rounded-lg border border-zinc-800 bg-zinc-900/30",
            div {
                class: "px-4 py-3 border-b border-zinc-800",
                h3 {
                    class: "text-sm font-semibold font-mono text-zinc-400",
                    "Footnotes"
                }
            }

            if footnotes().len() == 0 {
                div { class: "py-8 px-4 text-center",
                    p { class: "text-sm italic text-zinc-500",
                        "No footnotes found. Use [number] to add references."
                    }
                }
            }

            for footnote in footnotes() {
                div { class: "py-3 px-4 transition-colors group hover:bg-zinc-800/50",
                    div { class: "flex gap-3 items-center",
                        span { class: "flex-shrink-0 w-20 font-mono text-xs text-zinc-500",
                            "[{footnote.0}]"
                        }

                        if footnote.1.is_empty() {
                            {
                                let footnote_clone = footnote.clone();
                                rsx! {
                                    button {
                                        class: "flex flex-1 gap-2 items-center text-sm text-left transition-colors text-zinc-500 hover:text-zinc-300",
                                        onclick: move |_| {
                                            editing_footnote.set(
                                                Some(
                                                    (footnote_clone.0.clone(), String::new())
                                                )
                                            );
                                        },
                                        span { class: "italic", "Set link" }
                                    }
                                }
                            }
                        } else {
                            {
                                let footnote_clone = footnote.clone();
                                let footnote_clone2 = footnote.clone();
                                rsx! {
                                    button {
                                        class: "flex flex-1 gap-2 items-center text-sm text-left transition-colors text-zinc-300 hover:text-zinc-100",
                                        onclick: move |_| {
                                            tracing::info!("requesting nav to: {}", footnote.1);
                                            onlinkclick.call(footnote_clone.1.clone());
                                        },
                                        span { "{footnote.1}" }
                                    }
                                    button {
                                        class: "flex gap-2 items-center text-sm transition-colors text-zinc-500 hover:text-zinc-300",
                                        onclick: move |_| {
                                            editing_footnote.set(Some(footnote_clone2.clone()));
                                        },
                                        span { class: "text-xs", "Change" }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some((footnote_number, footnote_text)) = editing_footnote() {
                FootnoteEditor {
                    initial_value: (footnote_number, footnote_text),
                    onsave: move |(number, text): (String, String)| {
                        tracing::info!("footnote editor onsave: [{}] = {}", number, text);
                        onlink.call((number, text));
                        editing_footnote.set(None);
                    },
                    oncancel: move |_| editing_footnote.set(None),
                }
            }
        }
    }
}
