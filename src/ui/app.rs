use dioxus::prelude::*;
use std::path::PathBuf;
use crate::ui::markdown::SimpleMarkdown;

#[derive(Clone, Copy, PartialEq)]
enum Screen {
    Editor,
    Contacts,
}

#[derive(Clone, PartialEq)]
enum VaultStatus {
    Initializing,
    Ready(PathBuf),
    Error(String),
}

#[derive(Clone, PartialEq)]
struct OpenFile {
    path: PathBuf,
    filename: String,
    content: String,
    share_with: Vec<String>,
}

#[component]
pub fn App() -> Element {
    let mut current_screen = use_signal(|| Screen::Editor);
    let mut vault_status = use_signal(|| VaultStatus::Initializing);
    let mut open_file = use_signal(|| None::<OpenFile>);

    // Initialize vault and load home.md on first render
    use_effect(move || {
        spawn(async move {
            let home_dir = match dirs::home_dir() {
                Some(dir) => dir,
                None => {
                    vault_status.set(VaultStatus::Error("Could not find home directory".to_string()));
                    return;
                }
            };

            let vault_path = home_dir.join("footnotes");

            // Check if vault already exists
            let footnotes_dir = vault_path.join(".footnotes");
            if !footnotes_dir.exists() {
                // Initialize new vault
                match crate::core::init::init(
                    Some(vault_path.clone()),
                    Some("me"),
                    Some("primary")
                ).await {
                    Ok(_) => {},
                    Err(e) => {
                        vault_status.set(VaultStatus::Error(format!("Failed to initialize vault: {}", e)));
                        return;
                    }
                }
            }

            vault_status.set(VaultStatus::Ready(vault_path.clone()));

            // Load home.md
            let home_path = vault_path.join("notes").join("home.md");
            match crate::core::note::parse_note(&home_path) {
                Ok(note) => {
                    open_file.set(Some(OpenFile {
                        path: home_path.clone(),
                        filename: "home.md".to_string(),
                        content: note.content,
                        share_with: note.frontmatter.share_with,
                    }));
                },
                Err(e) => {
                    vault_status.set(VaultStatus::Error(format!("Failed to load home.md: {}", e)));
                }
            }
        });
    });

    rsx! {
        document::Stylesheet { href: asset!("/assets/tailwind.css") }

        div { class: "h-screen flex flex-col bg-gray-50",
            match vault_status() {
                VaultStatus::Initializing => rsx! {
                    div { class: "flex items-center justify-center h-full",
                        div { class: "text-center",
                            div { class: "text-lg font-medium text-gray-700", "Initializing vault..." }
                            div { class: "text-sm text-gray-500 mt-2", "Setting up ~/footnotes/" }
                        }
                    }
                },
                VaultStatus::Error(ref error) => rsx! {
                    div { class: "flex items-center justify-center h-full",
                        div { class: "text-center max-w-md",
                            div { class: "text-lg font-medium text-red-600", "Error" }
                            div { class: "text-sm text-gray-700 mt-2", "{error}" }
                        }
                    }
                },
                VaultStatus::Ready(ref _path) => rsx! {
                    // Navigation bar
                    nav { class: "bg-white border-b border-gray-200 px-4 py-3",
                        div { class: "flex gap-4",
                            button {
                                class: if current_screen() == Screen::Editor { "px-4 py-2 font-medium text-blue-600 border-b-2 border-blue-600" } else { "px-4 py-2 font-medium text-gray-600 hover:text-gray-900" },
                                onclick: move |_| current_screen.set(Screen::Editor),
                                "Editor"
                            }
                            button {
                                class: if current_screen() == Screen::Contacts { "px-4 py-2 font-medium text-blue-600 border-b-2 border-blue-600" } else { "px-4 py-2 font-medium text-gray-600 hover:text-gray-900" },
                                onclick: move |_| current_screen.set(Screen::Contacts),
                                "Contacts"
                            }
                        }
                    }

                    // Main content area
                    div { class: "flex-1 overflow-auto",
                        match current_screen() {
                            Screen::Editor => rsx! {
                                EditorScreen { open_file }
                            },
                            Screen::Contacts => rsx! {
                                ContactsScreen {}
                            },
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum EditorMode {
    View,
    Edit,
}

#[component]
fn EditorScreen(open_file: Signal<Option<OpenFile>>) -> Element {
    let mut edited_content = use_signal(|| String::new());
    let save_status = use_signal(|| String::new());
    let mut trigger_save = use_signal(|| false);
    let mut editor_mode = use_signal(|| EditorMode::View);

    // Initialize edited_content when file loads
    use_effect(move || {
        if let Some(ref file_data) = *open_file.read() {
            edited_content.set(file_data.content.clone());
        }
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
                            note.frontmatter.modified = chrono::Utc::now();

                            match crate::core::note::serialize_note(&note) {
                                Ok(serialized) => {
                                    match std::fs::write(&path, serialized) {
                                        Ok(_) => {
                                            save_status.set("Saved!".to_string());

                                            if let Some(mut file) = open_file.write().as_mut() {
                                                file.content = content;
                                            }

                                            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                                            save_status.set(String::new());
                                        }
                                        Err(e) => save_status.set(format!("Error: {}", e)),
                                    }
                                }
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
        let file_path = file_data.path.clone();
        let share_with = file_data.share_with.clone();

        rsx! {
            div { class: "max-w-4xl mx-auto p-6 h-full flex flex-col gap-4",
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
                                    // TODO: Handle link click - open/create file
                                    println!("Link clicked: {}", href);
                                }
                            }
                        }
                    } else {
                        // Edit mode: show textarea
                        label { class: "block text-sm font-medium text-gray-700 mb-2 flex-shrink-0", "Content" }
                        textarea {
                            class: "flex-1 w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono resize-none",
                            placeholder: "Start writing...",
                            value: "{edited_content}",
                            oninput: move |evt| edited_content.set(evt.value()),
                            onkeydown: move |evt| {
                                if (evt.modifiers().ctrl() || evt.modifiers().meta()) && evt.key() == Key::Character("s".to_string()) {
                                    evt.prevent_default();
                                    trigger_save.set(true);
                                }
                            },
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

#[component]
fn ContactsScreen() -> Element {
    rsx! {
        div { class: "max-w-4xl mx-auto p-6",
            div { class: "text-center text-gray-500 text-lg", "[ Contacts Placeholder ]" }
        }
    }
}
