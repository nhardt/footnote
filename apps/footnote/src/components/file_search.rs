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

        for entry in walkdir::WalkDir::new(&path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if let Some(file_name) = entry.file_name().to_str() {
                if !file_name.starts_with('.') && file_name.ends_with(".md") {
                    files.push(entry.path().to_path_buf());
                }
            }
        }

        let matcher = SkimMatcherV2::default();
        let mut results: Vec<(PathBuf, i64)> = files
            .iter()
            .filter_map(|path| {
                let display = path.file_stem()?.to_str()?;
                if query.is_empty() {
                    Some((path.clone(), 0))
                } else {
                    matcher
                        .fuzzy_match(display, &query)
                        .map(|score| (path.clone(), score))
                }
            })
            .collect();
        results.sort_by(|a, b| b.1.cmp(&a.1));
        results
            .into_iter()
            .map(|(p, _)| p)
            .take(10)
            .collect::<Vec<_>>()
    });

    rsx! {
        div { class: "file-search",
            input {
                r#type: "text",
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
                div { class: "search-dropdown",
                    for path in filtered_files() {
                        {
                            let file = path.to_string_lossy().to_string();
                            let file_path = search_path_clone.join(&file);
                            rsx! {
                                div {
                                    key: "{file}",
                                    class: "list-item",
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
