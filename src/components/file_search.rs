use dioxus::prelude::*;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::path::PathBuf;

#[component]
pub fn FileSearch(search_path: PathBuf, on_select: EventHandler<PathBuf>) -> Element {
    let mut search_input = use_signal(|| String::new());
    let mut dropdown_open = use_signal(|| false);
    let search_path_clone = search_path.clone();

    let filtered_files = use_memo(move || {
        let query = search_input();
        let path = search_path.clone();
        let mut files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&path) {
            for entry in entries.flatten() {
                if let Ok(file_name) = entry.file_name().into_string() {
                    if !file_name.starts_with('.') && file_name.ends_with(".md") {
                        files.push(file_name);
                    }
                }
            }
        }
        let matcher = SkimMatcherV2::default();
        let mut results: Vec<(String, i64)> = files
            .iter()
            .filter_map(|file| {
                if query.is_empty() {
                    Some((file.clone(), 0))
                } else {
                    matcher
                        .fuzzy_match(file, &query)
                        .map(|score| (file.clone(), score))
                }
            })
            .collect();
        results.sort_by(|a, b| b.1.cmp(&a.1));
        results
            .into_iter()
            .map(|(f, _)| f)
            .take(10)
            .collect::<Vec<_>>()
    });

    rsx! {
        div { class: "relative flex-shrink-0",
            input {
                r#type: "text",
                class: "w-full px-3 py-1.5 text-sm bg-zinc-800 border border-zinc-700 rounded-md text-zinc-100 placeholder-zinc-400 focus:outline-none focus:ring-2 focus:ring-indigo-600",
                placeholder: "Search files...",
                value: "{search_input}",
                onfocus: move |_| dropdown_open.set(true),
                oninput: move |evt| {
                    search_input.set(evt.value());
                    dropdown_open.set(!evt.value().is_empty());
                },
                onblur: move |_| {
                    // Delay hiding to allow click on dropdown
                    let mut dropdown_open = dropdown_open.clone();
                    spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                        dropdown_open.set(false);
                    });
                },
            }
            if dropdown_open() && !filtered_files().is_empty() {
                div { class: "absolute z-50 w-full mt-1 bg-zinc-800 border border-zinc-700 rounded-md shadow-lg max-h-60 overflow-y-auto",
                    for file in filtered_files() {
                        {
                            let file_path = search_path_clone.join(&file);
                            rsx! {
                                div {
                                    key: "{file}",
                                    class: "px-3 py-2 hover:bg-zinc-700 cursor-pointer text-sm text-zinc-200",
                                    onclick: move |_| {
                                        on_select.call(file_path.clone());
                                        search_input.set(String::new());
                                        dropdown_open.set(false);
                                    },
                                    "{file}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
