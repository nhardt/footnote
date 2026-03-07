use dioxus::prelude::*;

use std::collections::HashSet;
use std::path::Path;

use crate::context::app_context::AppContext;

#[component]
pub fn PathInput(
    value: ReadSignal<String>,
    placeholder: Option<String>,
    oninput: EventHandler<String>,
) -> Element {
    let app_context = use_context::<AppContext>();
    let mut show_suggestions = use_signal(|| false);
    let mut directories = use_signal(|| HashSet::<String>::new());

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
        let input = value.read();
        let dirs = directories.read();
        get_directory_completions(&dirs, &input)
    });

    rsx! {
        div { class: "relative",
            input {
                class: "w-full px-3 py-2 bg-zinc-950 border border-zinc-700 rounded-md text-sm font-mono focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500",
                placeholder: placeholder.unwrap_or_default(),
                r#type: "text",
                value: "{value}",
                onfocus: move |_| show_suggestions.set(true),
                onblur: move |_| show_suggestions.set(false),
                oninput: move |evt| {
                    show_suggestions.set(true);
                    oninput.call(evt.value())
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
                                        oninput.call(format!("{}/", s));
                                    },
                                    {suggestion}
                                }
                            }
                        }
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
