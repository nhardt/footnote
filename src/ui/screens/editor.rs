use crate::ui::context::VaultContext;
use crate::ui::plaintext::PlainTextViewer;
use dioxus::prelude::*;
use std::path::PathBuf;

#[derive(Clone, PartialEq)]
pub struct OpenFile {
    pub path: PathBuf,
    pub filename: String,
    pub content: String,
    pub share_with: Vec<String>,
    pub footnotes: Vec<crate::core::note::Footnote>,
}

#[derive(Clone, Copy, PartialEq)]
enum EditorMode {
    View,
    Edit,
}

#[component]
pub fn EditorScreen(open_file: Signal<Option<OpenFile>>) -> Element {
    let mut edited_content = use_signal(|| String::new());
    let mut edited_footnotes = use_signal(|| Vec::<crate::core::note::Footnote>::new());
    let save_status = use_signal(|| String::new());
    let mut trigger_save = use_signal(|| false);
    let mut editor_mode = use_signal(|| EditorMode::View);
    let mut last_loaded_path = use_signal(|| None::<PathBuf>);
    let mut editing_title = use_signal(|| false);
    let mut edited_title = use_signal(|| String::new());
    let mut footnote_selector_open = use_signal(|| false);
    let mut selected_footnote_number = use_signal(|| None::<usize>);
    let mut footnote_search_input = use_signal(|| String::new());
    let vault_ctx = use_context::<VaultContext>();

    use_effect(move || {
        if let Some(ref file_data) = *open_file.read() {
            edited_content.set(file_data.content.clone());
            edited_footnotes.set(file_data.footnotes.clone());
            last_loaded_path.set(Some(file_data.path.clone()));
        }
    });

    // Handle save when triggered
    use_effect(move || {
        if trigger_save() {
            trigger_save.set(false);

            if let Some(ref file_data) = *open_file.read() {
                let content = edited_content();
                let footnotes = edited_footnotes();
                let path = file_data.path.clone();
                let mut open_file = open_file.clone();
                let mut save_status = save_status.clone();

                spawn(async move {
                    save_status.set("Saving...".to_string());

                    match crate::core::note::parse_note(&path) {
                        Ok(mut note) => {
                            note.content = content.clone();
                            note.frontmatter.footnotes = footnotes.clone();
                            note.frontmatter.modified = crate::core::note::VectorTime::new(Some(
                                note.frontmatter.modified.clone(),
                            ));

                            match crate::core::note::serialize_note(&note) {
                                Ok(serialized) => match std::fs::write(&path, serialized) {
                                    Ok(_) => {
                                        save_status.set("Saved!".to_string());

                                        if let Some(file) = open_file.write().as_mut() {
                                            file.content = content;
                                            file.footnotes = footnotes;
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

        // Closure to save title rename
        let mut save_title = move || {
            let new_title = edited_title().trim().to_string();
            if new_title.is_empty() {
                editing_title.set(false);
                return;
            }

            // Validate filename (ASCII only, no path separators)
            if !new_title.chars().all(|c| c.is_ascii() && c != '/' && c != '\\') {
                editing_title.set(false);
                return;
            }

            let vault_path = match vault_ctx.get_vault() {
                Some(path) => path,
                None => {
                    editing_title.set(false);
                    return;
                }
            };

            if let Some(ref file_data) = *open_file.read() {
                let old_path = file_data.path.clone();
                let new_filename = format!("{}.md", new_title);
                let new_path = vault_path.join(&new_filename);

                let mut open_file = open_file.clone();
                let mut editing_title = editing_title.clone();

                spawn(async move {
                    if let Err(e) = std::fs::rename(&old_path, &new_path) {
                        tracing::warn!("Failed to rename file: {}", e);
                        editing_title.set(false);
                        return;
                    }

                    if let Some(file) = open_file.write().as_mut() {
                        file.path = new_path.clone();
                        file.filename = new_filename.clone();
                    }

                    let config = crate::ui::config::AppConfig {
                        last_vault_path: vault_path,
                        last_file: Some(new_filename),
                    };
                    if let Err(e) = config.save() {
                        tracing::warn!("Failed to save config: {}", e);
                    }

                    editing_title.set(false);
                });
            } else {
                editing_title.set(false);
            }
        };

        rsx! {
            div { class: "max-w-4xl mx-auto p-6 h-full flex flex-col gap-4",

                // Document title
                div { class: "flex-shrink-0",
                    if editing_title() {
                        // Edit mode: input field
                        input {
                            r#type: "text",
                            class: "w-full px-2 py-1 text-2xl font-bold text-app-text border-b-2 border-app-primary focus:outline-none bg-transparent",
                            value: "{edited_title}",
                            oninput: move |evt| edited_title.set(evt.value()),
                            onblur: move |_| save_title(),
                            onkeydown: move |evt| {
                                if evt.key() == Key::Enter {
                                    evt.prevent_default();
                                    save_title();
                                } else if evt.key() == Key::Escape {
                                    evt.prevent_default();
                                    editing_title.set(false);
                                }
                            },
                            autofocus: true,
                        }
                    } else {
                        // View mode: clickable title
                        div {
                            class: "text-2xl font-bold text-app-text cursor-pointer hover:text-app-primary-light px-2 py-1",
                            onclick: move |_| {
                                // Strip .md extension for editing
                                let title = filename.trim_end_matches(".md");
                                edited_title.set(title.to_string());
                                editing_title.set(true);
                            },
                            {
                                // Display without .md extension
                                filename.trim_end_matches(".md")
                            }
                        }
                    }
                }

                // Content area (view or edit mode)
                div { class: "flex-1 flex flex-col min-h-0",
                    if editor_mode() == EditorMode::View {
                        // View mode: render plain text with footnote highlighting
                        div { class: "flex-1 overflow-auto border border-app-border rounded-md bg-app-surface",
                            PlainTextViewer {
                                content: edited_content(),
                                footnotes: edited_footnotes(),
                                on_footnote_click: move |uuid: uuid::Uuid| {
                                    let vault_path = match vault_ctx.get_vault() {
                                        Some(path) => path,
                                        None => return,
                                    };

                                    let mut open_file = open_file.clone();
                                    let mut editor_mode = editor_mode.clone();

                                    spawn(async move {
                                        // Try to find existing note by UUID
                                        let file_path = match crate::core::note::find_note_by_uuid(&vault_path, &uuid) {
                                            Ok(Some(path)) => path,
                                            Ok(None) => {
                                                // Note doesn't exist - we'll need to create it
                                                // For now, just return and don't navigate
                                                // (Step 9 will handle creating new notes from footnotes)
                                                tracing::warn!("Footnote references UUID {} which doesn't exist", uuid);
                                                return;
                                            }
                                            Err(e) => {
                                                tracing::error!("Failed to search for UUID: {}", e);
                                                return;
                                            }
                                        };

                                        // Load the file
                                        match crate::core::note::parse_note(&file_path) {
                                            Ok(note) => {
                                                let filename = file_path
                                                    .file_name()
                                                    .and_then(|n: &std::ffi::OsStr| n.to_str())
                                                    .unwrap_or("unknown.md")
                                                    .to_string();

                                                open_file.set(Some(OpenFile {
                                                    path: file_path.clone(),
                                                    filename: filename.clone(),
                                                    content: note.content,
                                                    share_with: note.frontmatter.share_with,
                                                    footnotes: note.frontmatter.footnotes,
                                                }));

                                                // Switch to view mode
                                                editor_mode.set(EditorMode::View);

                                                // Save file to config
                                                let config = crate::ui::config::AppConfig {
                                                    last_vault_path: vault_path,
                                                    last_file: Some(filename),
                                                };
                                                if let Err(e) = config.save() {
                                                    tracing::warn!("Failed to save config: {}", e);
                                                }
                                            }
                                            Err(e) => {
                                                tracing::error!("Failed to load file: {}", e);
                                            }
                                        }
                                    });
                                }
                            }
                        }
                    } else {
                        label { class: "block text-sm font-medium text-app-text-secondary mb-2 flex-shrink-0", "Content" }
                        textarea {
                            class: "flex-1 w-full px-3 py-2 bg-app-surface border border-app-border rounded-md text-app-text placeholder-app-text-muted focus:outline-none focus:ring-2 focus:ring-app-primary font-mono resize-none",
                            placeholder: "Once upon a time...",
                            oninput: move |evt| {
                                let content = evt.value();
                                edited_content.set(content.clone());

                                // Auto-detect footnote references [1], [2], etc.
                                use regex::Regex;
                                let footnote_re = Regex::new(r"\[(\d+)\]").unwrap();
                                let mut found_numbers = std::collections::HashSet::new();

                                for cap in footnote_re.captures_iter(&content) {
                                    if let Ok(num) = cap[1].parse::<usize>() {
                                        found_numbers.insert(num);
                                    }
                                }

                                // Get current footnotes
                                let mut current_footnotes = edited_footnotes();

                                // Add new footnote entries for numbers that don't exist yet
                                for num in found_numbers {
                                    if !current_footnotes.iter().any(|f| f.number == num) {
                                        current_footnotes.push(crate::core::note::Footnote {
                                            number: num,
                                            title: String::new(),
                                            uuid: uuid::Uuid::nil(), // Placeholder UUID
                                        });
                                    }
                                }

                                // Sort by number
                                current_footnotes.sort_by_key(|f| f.number);

                                edited_footnotes.set(current_footnotes);
                            },
                            {edited_content},
                        }

                        // Footnotes section
                        if !edited_footnotes.read().is_empty() {
                            div { class: "flex-shrink-0 border-t border-app-border pt-4 mt-4",
                                label { class: "block text-sm font-medium text-app-text-secondary mb-2", "Footnotes" }
                                div { class: "space-y-2",
                                    {edited_footnotes.read().iter().map(|footnote| {
                                        let num = footnote.number;
                                        let title = footnote.title.clone();
                                        let uuid = footnote.uuid.to_string();
                                        let is_empty = title.is_empty();
                                        rsx! {
                                            div {
                                                key: "{num}",
                                                class: "flex items-center gap-3 px-3 py-2 bg-app-panel border border-app-border rounded-md text-sm hover:bg-app-hover cursor-pointer",
                                                onclick: move |_| {
                                                    selected_footnote_number.set(Some(num));
                                                    footnote_search_input.set(String::new());
                                                    footnote_selector_open.set(true);
                                                },
                                                div { class: "text-app-primary-light font-medium min-w-[3rem]", "[{num}]" }
                                                div {
                                                    class: if is_empty { "flex-1 text-app-text-muted italic" } else { "flex-1 text-app-text" },
                                                    if is_empty { "(click to set link)" } else { "{title}" }
                                                }
                                                div { class: "text-app-text-muted text-xs font-mono", "{uuid}" }
                                            }
                                        }
                                    })}
                                }
                            }
                        }
                    }
                }

                // Footnote selector modal
                if footnote_selector_open() {
                    div {
                        class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
                        onclick: move |_| footnote_selector_open.set(false),
                        div {
                            class: "bg-app-surface border border-app-border rounded-lg shadow-xl max-w-2xl w-full mx-4",
                            onclick: move |evt| evt.stop_propagation(),

                            // Header
                            div { class: "px-4 py-3 border-b border-app-border",
                                h3 { class: "text-lg font-medium text-app-text",
                                    "Select Note for [{selected_footnote_number.read().unwrap_or(0)}]"
                                }
                            }

                            // Search input
                            div { class: "p-4",
                                input {
                                    r#type: "text",
                                    class: "w-full px-3 py-2 bg-app-panel border border-app-border rounded-md text-app-text placeholder-app-text-muted focus:outline-none focus:ring-2 focus:ring-app-primary",
                                    placeholder: "Search files or enter new note title...",
                                    value: "{footnote_search_input}",
                                    oninput: move |evt| footnote_search_input.set(evt.value()),
                                    autofocus: true,
                                }
                            }

                            // File list
                            div { class: "max-h-96 overflow-y-auto px-4 pb-4",
                                {
                                    // Get vault path and scan files
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
                                            if footnote_search_input().is_empty() {
                                                Some((file.clone(), 0))
                                            } else {
                                                matcher
                                                    .fuzzy_match(file, &footnote_search_input())
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
                                            for file in filtered_files.iter() {
                                                {
                                                    let file = file.clone();
                                                    rsx! {
                                                        div {
                                                            key: "{file}",
                                                            class: "px-3 py-2 hover:bg-app-hover cursor-pointer rounded-md text-app-text-secondary",
                                                            onclick: move |_| {
                                                                let vault_path = match vault_ctx.get_vault() {
                                                                    Some(path) => path,
                                                                    None => return,
                                                                };
                                                                let file_path = vault_path.join(&file);
                                                                let file_title = file.trim_end_matches(".md").to_string();
                                                                let footnote_num = selected_footnote_number();
                                                                let mut edited_footnotes = edited_footnotes.clone();
                                                                let mut footnote_selector_open = footnote_selector_open.clone();

                                                                spawn(async move {
                                                                    // Load the file to get its UUID
                                                                    match crate::core::note::parse_note(&file_path) {
                                                                        Ok(note) => {
                                                                            if let Some(num) = footnote_num {
                                                                                // Update the footnote
                                                                                let mut footnotes = edited_footnotes();
                                                                                if let Some(footnote) = footnotes.iter_mut().find(|f| f.number == num) {
                                                                                    footnote.uuid = note.frontmatter.uuid;
                                                                                    footnote.title = file_title;
                                                                                }
                                                                                edited_footnotes.set(footnotes);
                                                                            }
                                                                            footnote_selector_open.set(false);
                                                                        }
                                                                        Err(e) => {
                                                                            tracing::error!("Failed to load file: {}", e);
                                                                        }
                                                                    }
                                                                });
                                                            },
                                                            "{file}"
                                                        }
                                                    }
                                                }
                                            }
                                        } else if !footnote_search_input().is_empty() {
                                            div { class: "px-3 py-2 text-app-text-muted text-sm",
                                                "No matching files found"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Bottom bar with Share and buttons
                div { class: "flex items-center justify-between gap-4 flex-shrink-0",
                    // Share with
                    div { class: "flex items-center gap-2",
                        label { class: "text-sm font-medium text-app-text-secondary", "Share With" }
                        div { class: "px-3 py-1.5 border border-app-border rounded-md bg-app-panel text-app-text-muted text-sm",
                            {
                                if share_with.is_empty() {
                                    "[ no contacts ]".to_string()
                                } else {
                                    share_with.join(", ")
                                }
                            }
                        }
                    }
                    // Edit/Save buttons
                    div { class: "flex items-center gap-2",
                        button {
                            class: if editor_mode() == EditorMode::Edit {
                                "px-4 py-2 bg-app-hover text-app-text-secondary rounded-md"
                            } else {
                                "px-4 py-2 bg-app-panel text-app-text-secondary border border-app-border rounded-md hover:bg-app-hover"
                            },
                            onclick: move |_| editor_mode.set(EditorMode::Edit),
                            "Edit"
                        }
                        if editor_mode() == EditorMode::Edit {
                            button {
                                class: "px-4 py-2 bg-app-primary text-white rounded-md hover:bg-app-primary-hover focus:outline-none focus:ring-2 focus:ring-app-primary",
                                onclick: move |_| {
                                    trigger_save.set(true);
                                    editor_mode.set(EditorMode::View);
                                },
                                "Save"
                            }
                        }
                        if !save_status().is_empty() {
                            div { class: "text-sm text-app-text-tertiary", "{save_status}" }
                        }
                    }
                }
            }
        }
    } else {
        rsx! {
            div { class: "max-w-4xl mx-auto p-6",
                div { class: "text-center text-app-text-muted", "Loading..." }
            }
        }
    }
}
