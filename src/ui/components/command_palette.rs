use crate::ui::context::VaultContext;
use crate::ui::screens::OpenFile;
use crate::ui::Screen;
use dioxus::prelude::*;
use std::path::PathBuf;

#[component]
pub fn CommandPalette(
    palette_input: Signal<String>,
    palette_open: Signal<bool>,
    current_file: Signal<Option<OpenFile>>,
    current_screen: Signal<Screen>,
) -> Element {
    let vault_ctx = use_context::<VaultContext>();

    rsx! {
        div { class: "relative flex-1 max-w-md mx-auto",
            input {
                r#type: "text",
                class: "w-full px-3 py-1.5 text-sm bg-zinc-800 border border-zinc-700 rounded-md text-zinc-100 placeholder-zinc-400 focus:outline-none focus:ring-2 focus:ring-indigo-600",
                placeholder: "Search files...",
                value: "{palette_input}",
                onfocus: move |_| palette_open.set(true),
                oninput: move |evt| {
                    palette_input.set(evt.value());
                    palette_open.set(!evt.value().is_empty());
                },
                onblur: move |_| {
                    // Delay hiding to allow click on dropdown
                    let mut palette_open = palette_open.clone();
                    spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                        palette_open.set(false);
                    });
                },
            }

            // Dropdown overlay
            if palette_open() {
                {
                    // Get vault path and scan files live
                    let vault_path = match vault_ctx.get_vault() {
                        Some(path) => path,
                        None => PathBuf::new(),
                    };

                    let mut files = Vec::new();
                    if !vault_path.as_os_str().is_empty() {
                        if let Ok(entries) = std::fs::read_dir(&vault_path) {
                            for entry in entries.flatten() {
                                if let Ok(file_name) = entry.file_name().into_string() {
                                    if !file_name.starts_with('.') && file_name.ends_with(".md") {
                                        files.push(file_name);
                                    }
                                }
                            }
                        }
                    }

                    // Fuzzy match files
                    use fuzzy_matcher::skim::SkimMatcherV2;
                    use fuzzy_matcher::FuzzyMatcher;

                    let matcher = SkimMatcherV2::default();
                    let mut filtered_files: Vec<(String, i64)> = files
                        .iter()
                        .filter_map(|file| {
                            if palette_input().is_empty() {
                                Some((file.clone(), 0))
                            } else {
                                matcher
                                    .fuzzy_match(file, &palette_input())
                                    .map(|score| (file.clone(), score))
                            }
                        })
                        .collect();
                    filtered_files.sort_by(|a, b| b.1.cmp(&a.1));
                    let filtered_files: Vec<String> = filtered_files
                        .into_iter()
                        .map(|(f, _)| f)
                        .take(10)
                        .collect();

                    rsx! {
                        if !filtered_files.is_empty() {
                            div { class: "absolute z-50 w-full mt-1 bg-zinc-800 border border-zinc-700 rounded-md shadow-lg max-h-60 overflow-y-auto",
                                for file in filtered_files.iter() {
                                    {
                                        let file = file.clone();
                                        rsx! {
                                            div {
                                                key: "{file}",
                                                class: "px-3 py-2 hover:bg-zinc-700 cursor-pointer text-sm text-zinc-200",
                                                onclick: move |_| {
                                                    let vault_path = match vault_ctx.get_vault() {
                                                        Some(path) => path,
                                                        None => return,
                                                    };
                                                    let file_path = vault_path.join(&file);
                                                    let file_name = file.clone();
                                                    let mut current_file = current_file.clone();
                                                    let mut current_screen = current_screen.clone();
                                                    let mut palette_input = palette_input.clone();
                                                    let mut palette_open = palette_open.clone();

                                                    spawn(async move {
                                                        match crate::core::note::parse_note(&file_path) {
                                                            Ok(note) => {
                                                                tracing::debug!("clicked on {} to open", file_name);
                                                                current_file.set(Some(OpenFile {
                                                                    path: file_path.clone(),
                                                                    filename: file_name.clone(),
                                                                    content: note.content,
                                                                    share_with: note.frontmatter.share_with,
                                                                    footnotes: note.frontmatter.footnotes,
                                                                }));
                                                                current_screen.set(Screen::Editor);
                                                                palette_input.set(String::new());
                                                                palette_open.set(false);

                                                                // Save file to config
                                                                let config = crate::ui::config::AppConfig {
                                                                    last_vault_path: vault_path,
                                                                    last_file: Some(file_name),
                                                                };
                                                                if let Err(e) = config.save() {
                                                                    tracing::warn!("Failed to save config: {}", e);
                                                                }
                                                            }
                                                            Err(e) => {
                                                                eprintln!("Failed to load file: {}", e);
                                                            }
                                                        }
                                                    });
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
        }
    }
}
