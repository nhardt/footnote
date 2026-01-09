use std::path::PathBuf;

use crate::{components::file_search::FileSearch, context::AppContext, model::note::Note};
use dioxus::prelude::*;
use indexmap::IndexMap;
use uuid::Uuid;

#[component]
pub fn Footnotes(
    footnotes: ReadSignal<IndexMap<String, String>>,
    onlink: EventHandler<(String, String)>,
    onuuidclick: EventHandler<String>,
) -> Element {
    let app_context = use_context::<AppContext>();
    let mut search_visible = use_signal(|| None::<String>);

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
                            "[^{footnote.0}]"
                        }

                        if footnote.1.is_empty() {
                            button {
                                class: "flex flex-1 gap-2 items-center text-sm text-left transition-colors text-zinc-500 hover:text-zinc-300",
                                onclick: move |_| {
                                    let name = footnote.0.clone();
                                    search_visible.set(Some(name));
                                },
                                span { class: "italic", "Set link" }
                            }
                        } else {
                            button {
                                class: "flex flex-1 gap-2 items-center text-sm text-left transition-colors text-zinc-300 hover:text-zinc-100",
                                onclick: move |_| {
                                    if let Some(uuid_part) = footnote.1.split("footnote://").nth(1) {
                                        tracing::info!("found uuid, requiesting to uuid: {}", uuid_part);
                                        onuuidclick.call(uuid_part.to_string());
                                    }
                                },
                                span { "{footnote.1}" }
                            }
                            button {
                                class: "flex gap-2 items-center text-sm transition-colors text-zinc-500 hover:text-zinc-300",
                                onclick: move |_| {
                                    let name = footnote.0.clone();
                                    search_visible.set(Some(name));
                                },
                                span { class: "text-xs", "Change" }
                            }
                        }
                    }
                }
            }

            if let Some(footnote_name) = search_visible() {
                div { class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50",
                    div { class: "bg-zinc-900 border border-zinc-700 rounded-lg p-4 w-96",
                        FileSearch {
                            search_path: app_context.vault.read().base_path(),
                            on_select: move |path: PathBuf| {
                                tracing::info!("link {} to {}", footnote_name, path.to_string_lossy());
                                let path_clone = path.clone();
                                let filename = path_clone.file_stem().unwrap().to_string_lossy().to_string();
                                if let Ok(note) = Note::from_path(path, false) {
                                    onlink.call((footnote_name.clone(), format!("{}|footnote://{}", filename, note.frontmatter.uuid)));
                                    search_visible.set(None);
                                };
                            }
                        }
                    }
                }
            }
        }
    }
}
