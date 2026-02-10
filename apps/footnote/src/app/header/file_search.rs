use dioxus::prelude::*;

use futures::stream::StreamExt;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::fs;
use std::path::{Component, PathBuf};

use crate::context::AppContext;
use crate::route::Route;

#[derive(Clone, Debug)]
struct SearchResult {
    path: PathBuf,
    score: i64,
    match_type: MatchType,
    preview: Option<String>,
}

#[derive(Clone, Debug)]
enum MatchType {
    Filename,
    Content { line_number: usize },
}

#[component]
pub fn FileSearch() -> Element {
    let app_context = use_context::<AppContext>();
    let search_path = app_context.vault.read().base_path();
    let vault_path = app_context.vault.read().base_path();

    let nav = use_navigator();

    let mut search_input = use_signal(|| String::new());
    let mut dropdown_open = use_signal(|| false);
    let mut search_results = use_signal(|| Vec::<SearchResult>::new());
    let mut is_searching = use_signal(|| false);

    let search_task = use_coroutine(move |mut rx: UnboundedReceiver<String>| {
        let search_path = search_path.clone();
        async move {
        while let Some(query) = rx.next().await {
            if query.is_empty() {
                search_results.set(Vec::new());
                continue;
            }

            is_searching.set(true);
            let path = search_path.clone();
            let matcher = SkimMatcherV2::default();
            let mut results = Vec::new();

            for entry in walkdir::WalkDir::new(&path)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if let Some(file_name) = entry.file_name().to_str() {
                    if !file_name.starts_with('.') && file_name.ends_with(".md") {
                        let path = entry.path().to_path_buf();
                        let display_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

                        if let Some(score) = matcher.fuzzy_match(display_name, &query) {
                            results.push(SearchResult {
                                path: path.clone(),
                                score,
                                match_type: MatchType::Filename,
                                preview: None,
                            });
                        }

                        if let Ok(content) = fs::read_to_string(&path) {
                            for (line_num, line) in content.lines().enumerate() {
                                if let Some(score) = matcher.fuzzy_match(line, &query) {
                                    let preview = line.trim().to_string();
                                    results.push(SearchResult {
                                        path: path.clone(),
                                        score: score / 2,
                                        match_type: MatchType::Content {
                                            line_number: line_num + 1,
                                        },
                                        preview: Some(preview),
                                    });
                                }
                            }
                        }
                    }
                }
            }

            results.sort_by(|a, b| b.score.cmp(&a.score));
            let top_results = results.into_iter().take(20).collect::<Vec<_>>();
            search_results.set(top_results);
            is_searching.set(false);
        }
    }
    });

    rsx! {
        div {
            class: "flex-1 relative",
            input {
                r#type: "text",
                class: "w-full px-3 py-1.5 bg-zinc-900 border border-zinc-700 rounded-md text-sm font-mono text-zinc-100 placeholder-zinc-500 focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500",
                placeholder: "Search notes, contacts...",
                value: "{search_input}",
                onfocus: move |_| dropdown_open.set(true),
                oninput: move |evt| {
                    let value = evt.value();
                    search_input.set(value.clone());
                    dropdown_open.set(!value.is_empty());
                    search_task.send(value);
                },
                onblur: move |_| {
                    dropdown_open.set(false);
                },
            }
            if dropdown_open() && !search_results().is_empty() {
                div {
                    class: "absolute top-full left-0 right-0 mt-1 bg-zinc-900 border border-zinc-700 rounded-md shadow-2xl z-50 max-h-96 overflow-y-auto",
                    onmousedown: move |e| e.prevent_default(),
                    for result in search_results() {
                        {
                            let file = result.path.to_string_lossy().to_string();
                            let Ok(relative_path) = result.path.strip_prefix(&vault_path) else {
                                return rsx!{};
                            };
                            let segments: Vec<String> = relative_path
                                .components()
                                .filter_map(|component| {
                                    match component {
                                        Component::Normal(os_str) => Some(os_str.to_string_lossy().into_owned()),
                                        _ => None,
                                    }
                                })
                                .collect();

                            let display_name = result.path.file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("(unnamed)");

                            rsx! {
                                button {
                                    key: "{file}-{result.match_type:?}",
                                    class: "w-full px-3 py-2 text-left hover:bg-zinc-800 border-b border-zinc-800 last:border-0",
                                    onclick: move |_| {
                                        nav.push(Route::NoteView { vault_relative_path_segments: segments.clone() });
                                        search_input.set(String::new());
                                        dropdown_open.set(false);
                                    },
                                    div {
                                        class: "flex flex-col gap-1",
                                        div {
                                            class: "flex items-center gap-3",
                                            span {
                                                class: "text-zinc-500 text-xs font-mono",
                                                match result.match_type {
                                                    MatchType::Filename => "File",
                                                    MatchType::Content { .. } => "Text",
                                                }
                                            }
                                            span {
                                                class: "text-sm text-zinc-100",
                                                "{display_name}.md"
                                            }
                                        }
                                        if let Some(preview) = &result.preview {
                                            div {
                                                class: "text-xs text-zinc-400 font-mono pl-12 truncate",
                                                "{preview}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if is_searching() {
                        div {
                            class: "px-3 py-2 text-xs text-zinc-500 text-center",
                            "Searching..."
                        }
                    }
                }
            }
        }
    }
}
