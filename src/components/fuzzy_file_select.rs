use crate::{context::AppContext, Route};
use dioxus::prelude::*;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::path::PathBuf;

#[component]
pub fn FuzzyFileSelect(onselect: EventHandler<PathBuf>, oncancel: EventHandler) -> Element {
    let mut app_context = use_context::<AppContext>();
    let mut search_input = use_signal(|| String::new());

    use_hook(|| {
        if let Err(e) = app_context.reload_manifest() {
            tracing::warn!("failed to reload manifest: {}", e);
        }
    });

    let filtered_files = use_memo(move || {
        let query = search_input();
        let manifest = app_context.manifest.read();

        let mut files: Vec<PathBuf> = manifest.values().map(|entry| entry.path.clone()).collect();

        if query.is_empty() {
            files.sort();
            return files.into_iter().take(50).collect::<Vec<_>>();
        }

        let matcher = SkimMatcherV2::default();
        let mut results: Vec<(PathBuf, i64)> = files
            .iter()
            .filter_map(|path| {
                let display = path.to_str()?;
                matcher
                    .fuzzy_match(display, &query)
                    .map(|score| (path.clone(), score))
            })
            .collect();

        results.sort_by(|a, b| b.1.cmp(&a.1));
        results
            .into_iter()
            .map(|(p, _)| p)
            .take(50)
            .collect::<Vec<_>>()
    });

    rsx! {
        div {
            class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-8 z-50",
            onclick: move |_| oncancel.call(()),

            div {
                class: "w-full max-w-2xl border border-zinc-700 rounded-lg bg-zinc-900 shadow-2xl flex flex-col max-h-[80vh]",
                onclick: move |evt| evt.stop_propagation(),

                div { class: "p-4 border-b border-zinc-800",
                    input {
                        class: "w-full px-4 py-2 bg-zinc-800 border border-zinc-700 rounded-md text-sm font-mono focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500",
                        r#type: "text",
                        placeholder: "Search files...",
                        value: "{search_input}",
                        oninput: move |evt| search_input.set(evt.value()),
                        autofocus: true,
                    }
                }

                div { class: "flex-1 overflow-y-auto",
                    for path in filtered_files() {
                        {
                            let path_clone = path.clone();
                            let display_path = path.to_string_lossy().to_string();
                            rsx! {
                                button {
                                    key: "{display_path}",
                                    class: "w-full px-4 py-2 text-left text-sm font-mono hover:bg-zinc-800 transition-colors border-b border-zinc-800/50",
                                    onclick: move |_| {
                                        onselect.call(path_clone.clone());
                                    },
                                    "{display_path}"
                                }
                            }
                        }
                    }
                }

                div { class: "p-4 border-t border-zinc-800 flex justify-end",
                    button {
                        class: "px-4 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all",
                        onclick: move |_| oncancel.call(()),
                        "Cancel"
                    }
                }
            }
        }
    }
}
