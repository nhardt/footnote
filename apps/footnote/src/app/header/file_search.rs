use dioxus::prelude::*;

use futures::stream::StreamExt;
use std::fs;
use std::path::{Component, PathBuf};

use crate::context::app_context::AppContext;
use crate::route::Route;

/// Split text into (before, match, after) segments for highlighting.
/// Returns None if query not found.
fn highlight_split(text: &str, query: &str) -> Option<(String, String, String)> {
    // mobile always wants to upcase the first letter, skip it for smart case
    // detection
    let case_sensitive = query.chars().skip(1).any(|c| c.is_uppercase());
    let start = if case_sensitive {
        text.find(query)?
    } else {
        text.to_lowercase().find(&query.to_lowercase())?
    };
    let end = start + query.len();
    Some((
        text[..start].to_string(),
        text[start..end].to_string(),
        text[end..].to_string(),
    ))
}

#[derive(Clone, Debug)]
struct SearchResult {
    path: PathBuf,
    display: String,
    match_type: MatchType,
    preview: Option<String>,
}

#[derive(Clone, Debug)]
enum MatchType {
    Filename,
    Content {},
}

#[component]
pub fn FileSearch() -> Element {
    let app_context = use_context::<AppContext>();
    let search_path = app_context.vault.read().base_path();
    let vault_path = app_context.vault.read().base_path();

    let nav = use_navigator();

    let mut search_input = use_signal(|| String::new());
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

                // file I/O occurs in blocking thread
                let results = tokio::task::spawn_blocking(move || {
                    let case_sensitive = query.chars().skip(1).any(|c| c.is_uppercase());
                    let query_normalized = if case_sensitive {
                        query.clone()
                    } else {
                        query.to_lowercase()
                    };
                    let mut results = Vec::new();

                    for entry in walkdir::WalkDir::new(&path)
                        .into_iter()
                        .filter_map(|e| e.ok())
                    {
                        if let Some(file_name) = entry.file_name().to_str() {
                            if !file_name.starts_with('.') && file_name.ends_with(".md") {
                                let mut match_count = 0;

                                let path = entry.path().to_path_buf();
                                let display_name = path
                                    .file_stem()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("")
                                    .to_string();

                                let name_cmp = if case_sensitive {
                                    display_name.clone()
                                } else {
                                    display_name.to_lowercase()
                                };
                                if name_cmp.contains(&query_normalized) {
                                    match_count += 1;
                                    results.push(SearchResult {
                                        path: path.clone(),
                                        display: display_name.clone(),
                                        match_type: MatchType::Filename,
                                        preview: None,
                                    });
                                }

                                if let Ok(content) = fs::read_to_string(&path) {
                                    for (_line_num, line) in content.lines().enumerate() {
                                        let line_cmp = if case_sensitive {
                                            line.to_string()
                                        } else {
                                            line.to_lowercase()
                                        };
                                        if line_cmp.contains(&query_normalized) {
                                            let preview = line.trim().to_string();
                                            results.push(SearchResult {
                                                path: path.clone(),
                                                display: display_name.clone(),
                                                match_type: MatchType::Content {},
                                                preview: Some(preview),
                                            });
                                            match_count += 1;
                                            if match_count > 2 {
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    results.into_iter().take(50).collect::<Vec<_>>()
                })
                .await
                .unwrap_or_default();

                search_results.set(results);
                is_searching.set(false);
            }
        }
    });

    rsx! {
        div {
            class: "flex-1",
            input {
                r#type: "text",
                class: "w-full px-3 py-1.5 bg-zinc-900 border border-zinc-700 rounded-md text-sm font-mono text-zinc-100 placeholder-zinc-500 focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500",
                placeholder: "Type to search",
                value: "{search_input}",
                oninput: move |evt| {
                    let value = evt.value();
                    is_searching.set(true);
                    search_input.set(value.clone());
                    search_task.send(value);
                },
            }

            if !search_input.is_empty() {
                div {
                    class: "overflow-y-auto absolute right-4 left-4 z-50 mt-1 max-h-96 rounded-md border shadow-2xl bg-zinc-900 border-zinc-700",
                    onmousedown: move |e| e.prevent_default(),

                    if !is_searching() && search_results().is_empty() {
                        div {
                            class: "px-3 py-2 text-xs text-zinc-500 text-center",
                            "No results found"
                        }
                    }
                    else if is_searching() && search_results().is_empty() {
                        div {
                            class: "px-3 py-2 text-xs text-zinc-500 text-center",
                            "Searching..."
                        }
                    }
                    else {
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

                                let display_name = result.display.clone();
                                let query = search_input();
                                let preview = match &result.match_type {
                                    MatchType::Filename => None,
                                    MatchType::Content { .. } => result.preview.clone(),
                                };
                                let name_parts = highlight_split(&display_name, &query);
                                let preview_parts = preview.as_ref().and_then(|p| highlight_split(p, &query));

                                rsx! {
                                    button {
                                        key: "{file}-{result.match_type:?}",
                                        class: "w-full px-2 py-1 text-left hover:bg-zinc-800 border-b border-zinc-800/50 last:border-0 font-mono text-sm flex items-center gap-2",
                                        onclick: move |_| {
                                            nav.push(Route::NoteView { vault_relative_path_segments: segments.clone() });
                                            search_input.set(String::new());
                                        },
                                        span {
                                            class: "text-zinc-500 text-xs shrink-0",
                                            if let Some((before, matched, after)) = &name_parts {
                                                span { "{before}" }
                                                span { class: "text-yellow-400", "{matched}" }
                                                span { "{after}" }
                                            } else {
                                                span { "{display_name}" }
                                            }
                                        }
                                        if let Some(text) = &preview {
                                            span {
                                                class: "text-zinc-100 truncate",
                                                if let Some((before, matched, after)) = &preview_parts {
                                                    span { "{before}" }
                                                    span { class: "text-yellow-400", "{matched}" }
                                                    span { "{after}" }
                                                } else {
                                                    span { "{text}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
