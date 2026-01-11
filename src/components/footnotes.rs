use crate::{
    components::fuzzy_file_select::FuzzyFileSelect, context::AppContext, model::note::Note,
};
use dioxus::prelude::*;
use indexmap::IndexMap;
use std::path::PathBuf;

#[component]
pub fn Footnotes(
    footnotes: ReadSignal<IndexMap<String, String>>,
    onlink: EventHandler<(String, String)>,
    onuuidclick: EventHandler<String>,
) -> Element {
    let app_context = use_context::<AppContext>();
    let mut editing_footnote = use_signal(|| None::<String>);

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
                            button {
                                class: "flex flex-1 gap-2 items-center text-sm text-left transition-colors text-zinc-500 hover:text-zinc-300",
                                onclick: move |_| {
                                    editing_footnote.set(Some(footnote.0.clone()));
                                },
                                span { class: "italic", "Set link" }
                            }
                        } else {
                            button {
                                class: "flex flex-1 gap-2 items-center text-sm text-left transition-colors text-zinc-300 hover:text-zinc-100",
                                onclick: move |_| {
                                    if let Some(uuid_part) = footnote.1.split("footnote://").nth(1) {
                                        tracing::info!("found uuid, requesting nav to: {}", uuid_part);
                                        onuuidclick.call(uuid_part.to_string());
                                    }
                                },
                                span { "{footnote.1}" }
                            }
                            button {
                                class: "flex gap-2 items-center text-sm transition-colors text-zinc-500 hover:text-zinc-300",
                                onclick: move |_| {
                                    editing_footnote.set(Some(footnote.0.clone()));
                                },
                                span { class: "text-xs", "Change" }
                            }
                        }
                    }
                }
            }

            if let Some(footnote_name) = editing_footnote() {
                FuzzyFileSelect {
                    onselect: move |path: PathBuf| {
                        tracing::info!("linking [{}] to {}", footnote_name, path.display());
                        let full_path = app_context.vault.read().base_path().join(&path);
                        let filename = path.file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();

                        if let Ok(note) = Note::from_path(full_path, false) {
                            let link_value = format!("{}|footnote://{}", filename, note.frontmatter.uuid);
                            onlink.call((footnote_name.clone(), link_value));
                        }
                        editing_footnote.set(None);
                    },
                    oncancel: move |_| editing_footnote.set(None),
                }
            }
        }
    }
}
