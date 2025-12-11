use crate::ui::context::VaultContext;
use crate::ui::markdown::SimpleMarkdown;
use dioxus::prelude::*;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::path::PathBuf;

#[derive(Clone, PartialEq)]
pub struct OpenFile {
    pub path: PathBuf,
    pub filename: String,
    pub content: String,
    pub share_with: Vec<String>,
}

#[derive(Clone, Copy, PartialEq)]
enum EditorMode {
    View,
    Edit,
}

#[component]
pub fn EditorScreen(open_file: Signal<Option<OpenFile>>) -> Element {
    let mut edited_content = use_signal(|| String::new());
    let save_status = use_signal(|| String::new());
    let mut trigger_save = use_signal(|| false);
    let mut editor_mode = use_signal(|| EditorMode::View);
    let mut all_files = use_signal(|| Vec::<String>::new());
    let mut picker_input = use_signal(|| String::new());
    let mut show_dropdown = use_signal(|| false);
    let mut last_loaded_path = use_signal(|| None::<PathBuf>);
    let vault_ctx = use_context::<VaultContext>();

    use_effect(move || {
        if let Some(ref file_data) = *open_file.read() {
            edited_content.set(file_data.content.clone());
            last_loaded_path.set(Some(file_data.path.clone()));
        }
    });

    // Scan notes directory for all markdown files on mount
    use_effect(move || {
        let vault_ctx = vault_ctx.clone();
        spawn(async move {
            let vault_path = match vault_ctx.get_vault() {
                Some(path) => path,
                None => return,
            };

            if let Ok(entries) = std::fs::read_dir(vault_path) {
                let mut files = Vec::new();
                for entry in entries.flatten() {
                    if let Ok(file_name) = entry.file_name().into_string() {
                        // Skip directories starting with "." (like .footnotes, .obsidian)
                        if file_name.starts_with('.') {
                            continue;
                        }
                        if file_name.ends_with(".md") {
                            files.push(file_name);
                        }
                    }
                }
                files.sort();
                all_files.set(files);
            }
        });
    });

    // Handle save when triggered
    use_effect(move || {
        if trigger_save() {
            trigger_save.set(false);

            if let Some(ref file_data) = *open_file.read() {
                let content = edited_content();
                let path = file_data.path.clone();
                let mut open_file = open_file.clone();
                let mut save_status = save_status.clone();

                spawn(async move {
                    save_status.set("Saving...".to_string());

                    match crate::core::note::parse_note(&path) {
                        Ok(mut note) => {
                            note.content = content.clone();
                            note.frontmatter.modified = crate::core::note::VectorTime::new(Some(
                                note.frontmatter.modified.clone(),
                            ));

                            match crate::core::note::serialize_note(&note) {
                                Ok(serialized) => match std::fs::write(&path, serialized) {
                                    Ok(_) => {
                                        save_status.set("Saved!".to_string());

                                        if let Some(file) = open_file.write().as_mut() {
                                            file.content = content;
                                        }

                                        tokio::time::sleep(tokio::time::Duration::from_secs(2))
                                            .await;
                                        save_status.set(String::new());
                                    }
                                    Err(e) => save_status.set(format!("Error: {}", e)),
                                },
                                Err(e) => save_status.set(format!("Error: {}", e)),
                            }
                        }
                        Err(e) => save_status.set(format!("Error: {}", e)),
                    }
                });
            }
        }
    });

    let file = open_file.read();

    if let Some(ref file_data) = *file {
        let filename = file_data.filename.clone();
        let share_with = file_data.share_with.clone();

        // Compute fuzzy-matched files
        let matcher = SkimMatcherV2::default();
        let mut filtered_files: Vec<(String, i64)> = all_files()
            .iter()
            .filter_map(|file| {
                if picker_input().is_empty() {
                    Some((file.clone(), 0))
                } else {
                    matcher
                        .fuzzy_match(file, &picker_input())
                        .map(|score| (file.clone(), score))
                }
            })
            .collect();
        filtered_files.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by score descending
        let filtered_files: Vec<String> = filtered_files
            .into_iter()
            .map(|(f, _)| f)
            .take(10)
            .collect();

        rsx! {
            div { class: "max-w-4xl mx-auto p-6 h-full flex flex-col gap-4",
                // File picker
                div { class: "relative flex-shrink-0",
                    label { class: "block text-sm font-medium text-gray-700 mb-2", "Open File" }
                    input {
                        r#type: "text",
                        class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                        placeholder: "Type to search files...",
                        value: "{picker_input}",
                        oninput: move |evt| {
                            picker_input.set(evt.value());
                            show_dropdown.set(!evt.value().is_empty());
                        },
                        onfocus: move |_| {
                            if !picker_input().is_empty() {
                                show_dropdown.set(true);
                            }
                        },
                        onblur: move |_| {
                            // Delay hiding to allow click on dropdown
                            let mut show_dropdown = show_dropdown.clone();
                            spawn(async move {
                                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                                show_dropdown.set(false);
                            });
                        },
                    }
                    // Dropdown
                    if show_dropdown() && !filtered_files.is_empty() {
                        div { class: "absolute z-10 w-full mt-1 bg-white border border-gray-300 rounded-md shadow-lg max-h-60 overflow-y-auto",
                            for file in filtered_files.iter() {
                                {
                                    let file = file.clone();
                                    rsx! {
                                        div {
                                            key: "{file}",
                                            class: "px-3 py-2 hover:bg-blue-50 cursor-pointer",
                                            onclick: move |_| {
                                                let vault_path = match vault_ctx.get_vault() {
                                                    Some(path) => path,
                                                    None => return,
                                                };
                                                let file_path = vault_path.join(&file);
                                                let file_name = file.clone();
                                                let mut open_file = open_file.clone();
                                                let mut editor_mode = editor_mode.clone();
                                                let mut picker_input = picker_input.clone();
                                                let mut show_dropdown = show_dropdown.clone();

                                                spawn(async move {
                                                    match crate::core::note::parse_note(&file_path) {
                                                        Ok(note) => {
                                                            open_file.set(Some(OpenFile {
                                                                path: file_path.clone(),
                                                                filename: file_name.clone(),
                                                                content: note.content,
                                                                share_with: note.frontmatter.share_with,
                                                            }));
                                                            editor_mode.set(EditorMode::View);
                                                            picker_input.set(String::new());
                                                            show_dropdown.set(false);

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

                // Document title/filename and buttons
                div { class: "flex items-end justify-between gap-4 flex-shrink-0",
                    div { class: "flex-1",
                        label { class: "block text-sm font-medium text-gray-700 mb-2", "Document" }
                        div { class: "px-3 py-2 border border-gray-300 rounded-md bg-gray-50 text-gray-900 font-mono",
                            "{filename}"
                        }
                    }
                    div { class: "flex items-center gap-2",
                        // View/Edit toggle
                        button {
                            class: if editor_mode() == EditorMode::View {
                                "px-4 py-2 bg-gray-200 text-gray-700 rounded-md"
                            } else {
                                "px-4 py-2 bg-white text-gray-700 border border-gray-300 rounded-md hover:bg-gray-50"
                            },
                            onclick: move |_| editor_mode.set(EditorMode::View),
                            "View"
                        }
                        button {
                            class: if editor_mode() == EditorMode::Edit {
                                "px-4 py-2 bg-gray-200 text-gray-700 rounded-md"
                            } else {
                                "px-4 py-2 bg-white text-gray-700 border border-gray-300 rounded-md hover:bg-gray-50"
                            },
                            onclick: move |_| editor_mode.set(EditorMode::Edit),
                            "Edit"
                        }
                        // Save button (only visible in edit mode)
                        if editor_mode() == EditorMode::Edit {
                            button {
                                class: "px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500",
                                onclick: move |_| {
                                    trigger_save.set(true);
                                },
                                "Save"
                            }
                        }
                        if !save_status().is_empty() {
                            div { class: "text-sm text-gray-600", "{save_status}" }
                        }
                    }
                }

                // Share with
                div { class: "flex-shrink-0",
                    label { class: "block text-sm font-medium text-gray-700 mb-2", "Share With" }
                    div { class: "px-3 py-2 border border-gray-300 rounded-md bg-gray-50 text-gray-500",
                        {
                            if share_with.is_empty() {
                                "[ no contacts ]".to_string()
                            } else {
                                share_with.join(", ")
                            }
                        }
                    }
                }

                // Content area (view or edit mode)
                div { class: "flex-1 flex flex-col min-h-0",
                    if editor_mode() == EditorMode::View {
                        // View mode: render markdown
                        div { class: "flex-1 overflow-auto border border-gray-300 rounded-md bg-white",
                            SimpleMarkdown {
                                content: edited_content(),
                                on_internal_link_click: move |href: String| {
                                    let vault_path = match vault_ctx.get_vault() {
                                        Some(path) => path,
                                        None => return,
                                    };

                                    let file_path = vault_path.join(&href);
                                    let mut open_file = open_file.clone();
                                    let mut editor_mode = editor_mode.clone();

                                    spawn(async move {
                                        // Check if file exists
                                        if !file_path.exists() {
                                            // Create new file with basic frontmatter
                                            let uuid = uuid::Uuid::new_v4();
                                            let vector_time = crate::core::note::VectorTime::default();
                                            let new_content = format!(
                                                r#"---
uuid: {}
modified: {}
share_with: []
---

# {}
"#,
                                                uuid,
                                                vector_time.as_i64(),
                                                href.trim_end_matches(".md")
                                            );

                                            if let Err(e) = std::fs::write(&file_path, new_content) {
                                                eprintln!("Failed to create file: {}", e);
                                                return;
                                            }
                                        }

                                        // Load the file
                                        match crate::core::note::parse_note(&file_path) {
                                            Ok(note) => {
                                                open_file.set(Some(OpenFile {
                                                    path: file_path.clone(),
                                                    filename: href.clone(),
                                                    content: note.content,
                                                    share_with: note.frontmatter.share_with,
                                                }));
                                                // Switch to view mode
                                                editor_mode.set(EditorMode::View);

                                                // Save file to config
                                                let config = crate::ui::config::AppConfig {
                                                    last_vault_path: vault_path,
                                                    last_file: Some(href.clone()),
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
                                }
                            }
                        }
                    } else {
                        label { class: "block text-sm font-medium text-gray-700 mb-2 flex-shrink-0", "Content" }
                        textarea {
                            class: "flex-1 w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono resize-none",
                            placeholder: "Once upon a time...",
                            onchange: move |evt| {
                                edited_content.set(evt.value());
                            },
                            {edited_content},
                        }
                    }
                }
            }
        }
    } else {
        rsx! {
            div { class: "max-w-4xl mx-auto p-6",
                div { class: "text-center text-gray-500", "Loading..." }
            }
        }
    }
}
