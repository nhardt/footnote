use dioxus::prelude::*;

use chrono::Local;
use std::collections::HashSet;
use std::path::Path;

use crate::context::app_context::AppContext;
use crate::context::menu_context::MenuContext;

#[component]
pub fn NewNoteModal() -> Element {
    let app_context = use_context::<AppContext>();
    let menu_context = use_context::<MenuContext>();
    let mut show_suggestions = use_signal(|| false);
    let mut directories = use_signal(|| HashSet::<String>::new());
    let mut note_path = use_signal(|| {
        menu_context
            .new_note_path_prefix
            .read()
            .clone()
            .unwrap_or_default()
    });
    let mut error_message = use_signal(|| String::new());

    use_effect(move || {
        let vault_path = app_context.vault.read().base_path();
        spawn(async move {
            let dirs = tokio::task::spawn_blocking(move || scan_directories(&vault_path))
                .await
                .unwrap_or_default();
            directories.set(dirs);
        });
    });

    let suggestions = use_memo(move || {
        let input = note_path.read();
        let dirs = directories.read();
        get_directory_completions(&dirs, &input)
    });

    let create_by_path = move |_| async move {
        let path_str = note_path.read().trim().to_string();
        if path_str.is_empty() {
            error_message.set("Path cannot be empty".to_string());
            return;
        }

        let full_path = app_context.vault.read().base_path().join(&path_str);

        if let Some(parent) = full_path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                error_message.set(format!("Failed to create directory: {}", e));
                return;
            }
        }

        if let Err(e) = app_context.vault.read().note_create(&full_path, "") {
            error_message.set(format!("Failed to create note: {}", e));
            return;
        }

        consume_context::<MenuContext>().go_note(&path_str);
    };

    let create_now_note = move |_| async move {
        let now = Local::now();
        let path_str = now.format("%Y/%m/%d/%H_%M_%S.md").to_string();
        let full_path = app_context.vault.read().base_path().join(&path_str);

        if let Some(parent) = full_path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                error_message.set(format!("Failed to create directory: {}", e));
                return;
            }
        }

        if let Err(e) = app_context.vault.read().note_create(&full_path, "") {
            error_message.set(format!("Failed to create note: {}", e));
            return;
        }

        consume_context::<MenuContext>().go_note(&path_str);
    };

    rsx! {
        div {
            class: "fixed text-zinc-100 inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50",
            onclick: move |_| consume_context::<MenuContext>().close_all(),

            div {
                class: "bg-zinc-900 border border-zinc-800 rounded-lg shadow-2xl max-w-md w-full",
                onclick: move |evt| evt.stop_propagation(),

                div { class: "p-6 border-b border-zinc-800",
                    h2 { class: "text-lg font-semibold", "Create New Note" }
                }

                div { class: "p-6 space-y-6",
                    div { class: "space-y-3",
                        label { class: "block text-sm font-medium text-zinc-300",
                            "Create by path"
                        }
                        div { class: "flex gap-2",
                            div { class: "flex-1 relative",
                            input {
                                class: "flex-1 px-3 py-2 bg-zinc-950 border border-zinc-700 rounded-md text-sm font-mono focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500",
                                placeholder: "path/to/note.md",
                                r#type: "text",
                                value: "{note_path}",
                                onfocus: move |_| show_suggestions.set(true),
                                onblur: move |_| show_suggestions.set(false),
                                oninput: move |evt| {
                                    note_path.set(evt.value());
                                    error_message.set(String::new());
                                    show_suggestions.set(true);
                                },
                            }
                            if show_suggestions() && !suggestions.is_empty() {
                                div {
                                    class: "absolute top-full left-0 right-0 mt-1 bg-zinc-950 border border-zinc-700 rounded-md shadow-2xl z-50 max-h-48 overflow-y-auto",
                                    onmousedown: move |e| e.prevent_default(),
                                    for suggestion in suggestions() {
                                        {
                                            let s = suggestion.clone();
                                            rsx! {
                                                button {
                                                    class: "w-full px-3 py-2 text-left text-sm font-mono text-zinc-300 hover:bg-zing-800 border-b border-zinc-800/50 last:border-0",
                                                    onclick: move |_| {
                                                            note_path.set(format!("{}/", s));
                                                    },
                                                    {suggestion}
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            button {
                                class: "px-4 py-2 bg-zinc-100 hover:bg-white hover:shadow-lg text-zinc-900 rounded-md text-sm font-medium transition-all",
                                disabled: note_path.read().trim().is_empty(),
                                onclick: create_by_path,
                                "Create"
                            }
                            }
                        }
                        if !error_message.read().is_empty() {
                            p { class: "text-sm text-red-400",
                                "{error_message}"
                            }
                        }
                    }

                    div { class: "relative",
                        div { class: "absolute inset-0 flex items-center",
                            div { class: "w-full border-t border-zinc-700" }
                        }
                        div { class: "relative flex justify-center text-xs uppercase",
                            span { class: "bg-zinc-900 px-2 text-zinc-500", "or" }
                        }
                    }

                    div { class: "space-y-3",
                        label { class: "block text-sm font-medium text-zinc-300",
                            "Quick capture"
                        }
                        button {
                            class: "w-full px-4 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all text-left flex items-center gap-2",
                            onclick: create_now_note,
                            svg {
                                class: "w-4 h-4 text-zinc-400",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    d: "M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                }
                            }
                            span { "Create timestamped note" }
                        }
                        p { class: "text-xs text-zinc-500",
                            "Creates: {Local::now().format(\"%Y/%m/%d/%H_%M_%S.md\")}"
                        }
                    }
                }

                div { class: "p-6 border-t border-zinc-800",
                    button {
                        class: "w-full px-4 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all",
                        onclick: move |_| consume_context::<MenuContext>().close_all(),
                        "Cancel"
                    }
                }
            }
        }
    }
}

fn get_directory_completions(directory_paths: &HashSet<String>, input: &str) -> Vec<String> {
    let input = input.strip_prefix("/").unwrap_or(input);

    let mut suggestions: Vec<String> = directory_paths
        .iter()
        .filter(|p| !p.is_empty() && p.to_lowercase().starts_with(input))
        .cloned()
        .collect();

    suggestions.sort_by(|a, b| {
        let depth_a = a.matches("/").count();
        let depth_b = b.matches("/").count();
        depth_a.cmp(&depth_b).then_with(|| a.cmp(b))
    });

    suggestions.truncate(10);
    suggestions
}

fn scan_directories(vault_path: &Path) -> HashSet<String> {
    let mut ret = HashSet::<String>::new();
    for entry in walkdir::WalkDir::new(vault_path)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_str().unwrap_or("");
            if name.starts_with(".") {
                return false;
            }
            if name == "footnotes" && e.depth() == 1 {
                return false;
            }
            true
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
    {
        if let Ok(rel) = entry.path().strip_prefix(vault_path) {
            let s = rel.to_string_lossy().to_string();
            ret.insert(s);
        }
    }

    ret
}
