use crate::context::AppContext;
use footnote_core::model::note::Note;
use dioxus::prelude::*;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::path::PathBuf;

#[component]
pub fn FootnoteEditor(
    initial_value: (String, String),
    onsave: EventHandler<(String, String)>,
    oncancel: EventHandler,
) -> Element {
    let mut app_context = use_context::<AppContext>();
    let footnote_number = use_signal(|| initial_value.0.clone());
    let mut footnote_text = use_signal(|| initial_value.1.clone());
    let mut search_input = use_signal(|| String::new());

    use_hook(|| {
        // todo: reload this on new file or at a time when new files have
        // arrived, or an inotify style event
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

    let mut handle_file_select = move |path: PathBuf| {
        let full_path = app_context.vault.read().base_path().join(&path);
        let filename = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        if let Ok(note) = Note::from_path(full_path, true) {
            let link_value = format!("{}|footnote://{}", filename, note.frontmatter.uuid);
            footnote_text.set(link_value);
            search_input.set(String::new());
        }
    };

    let handle_save = move |_| {
        onsave.call((footnote_number(), footnote_text()));
    };

    rsx! {
        div {
            class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-8 z-50",
            onclick: move |_| oncancel.call(()),

            div {
                class: "w-full max-w-2xl border border-zinc-700 rounded-lg bg-zinc-900 shadow-2xl flex flex-col",
                onclick: move |evt| evt.stop_propagation(),

                // Header
                div { class: "p-4 border-b border-zinc-800",
                    h3 { class: "text-sm font-semibold text-zinc-300",
                        "Edit Footnote Number [{initial_value.0}]"
                    }
                }

                // Result text field with Save button
                div { class: "p-4 border-b border-zinc-800",
                    div { class: "flex gap-2",
                        input {
                            class: "flex-1 px-4 py-2 bg-zinc-800 border border-zinc-700 rounded-md text-sm font-mono focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500",
                            r#type: "text",
                            placeholder: "Paste URL, type text, or select a file below...",
                            value: "{footnote_text}",
                            oninput: move |evt| footnote_text.set(evt.value()),
                            autofocus: true,
                        }
                        button {
                            class: "px-6 py-2 bg-blue-600 hover:bg-blue-500 rounded-md text-sm font-medium transition-colors",
                            onclick: handle_save,
                            "Save"
                        }
                    }
                }

                // File search input
                div { class: "p-4 border-b border-zinc-800",
                    input {
                        class: "w-full px-4 py-2 bg-zinc-800 border border-zinc-700 rounded-md text-sm font-mono focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500",
                        r#type: "text",
                        placeholder: "Search files...",
                        value: "{search_input}",
                        oninput: move |evt| search_input.set(evt.value()),
                    }
                }

                // Fuzzy results
                if !filtered_files().is_empty() {
                    div { class: "max-h-96 overflow-y-auto",
                        for path in filtered_files() {
                            {
                                let path_clone = path.clone();
                                let display_path = path.to_string_lossy().to_string();
                                rsx! {
                                    button {
                                        key: "{display_path}",
                                        class: "w-full px-4 py-2 text-left text-sm font-mono hover:bg-zinc-800 transition-colors border-b border-zinc-800/50",
                                        onclick: move |_| {
                                            handle_file_select(path_clone.clone());
                                        },
                                        "{display_path}"
                                    }
                                }
                            }
                        }
                    }
                }

                // Footer with Cancel
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
