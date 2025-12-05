use dioxus::prelude::*;
use std::path::PathBuf;
use crate::ui::markdown::SimpleMarkdown;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

#[derive(Clone, Copy, PartialEq)]
enum Screen {
    Editor,
    Contacts,
}

#[derive(Clone, PartialEq)]
enum VaultStatus {
    Initializing,
    NeedsSetup,
    SettingUpPrimary { device_name: String },
    SettingUpSecondary { device_name: String, connect_url: String },
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
    let reload_trigger = use_signal(|| 0);

    // Initialize vault and load home file on first render or when reload_trigger changes
    use_effect(move || {
        let _ = reload_trigger(); // Watch for changes
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
                // Need setup - show setup screen
                vault_status.set(VaultStatus::NeedsSetup);
                return;
            }

            // Set the vault as working directory for the entire app
            if let Err(e) = std::env::set_current_dir(&vault_path) {
                vault_status.set(VaultStatus::Error(format!("Failed to set working directory: {}", e)));
                return;
            }

            // Get the local device name to load correct home file
            let device_name = match crate::core::device::get_local_device_name() {
                Ok(name) => name,
                Err(e) => {
                    vault_status.set(VaultStatus::Error(format!("Failed to get device name: {}", e)));
                    return;
                }
            };

            vault_status.set(VaultStatus::Ready(vault_path.clone()));

            // Load device-specific home file
            let home_filename = format!("home-{}.md", device_name);
            let home_path = vault_path.join("notes").join(&home_filename);
            match crate::core::note::parse_note(&home_path) {
                Ok(note) => {
                    open_file.set(Some(OpenFile {
                        path: home_path.clone(),
                        filename: home_filename,
                        content: note.content,
                        share_with: note.frontmatter.share_with,
                    }));
                },
                Err(e) => {
                    vault_status.set(VaultStatus::Error(format!("Failed to load {}: {}", home_filename, e)));
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
                VaultStatus::NeedsSetup => rsx! {
                    SetupScreen { vault_status, reload_trigger }
                },
                VaultStatus::SettingUpPrimary { ref device_name } => rsx! {
                    div { class: "flex items-center justify-center h-full",
                        div { class: "text-center",
                            div { class: "text-lg font-medium text-gray-700", "Setting up primary device..." }
                            div { class: "text-sm text-gray-500 mt-2", "Device: {device_name}" }
                        }
                    }
                },
                VaultStatus::SettingUpSecondary { ref device_name, .. } => rsx! {
                    div { class: "flex items-center justify-center h-full",
                        div { class: "text-center",
                            div { class: "text-lg font-medium text-gray-700", "Connecting to existing vault..." }
                            div { class: "text-sm text-gray-500 mt-2", "Device: {device_name}" }
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
enum SetupMode {
    ChooseMode,
    ConnectToExisting,
}

#[component]
fn SetupScreen(vault_status: Signal<VaultStatus>, reload_trigger: Signal<i32>) -> Element {
    let mut device_name = use_signal(|| String::new());
    let mut connect_url = use_signal(|| String::new());
    let mut setup_mode = use_signal(|| SetupMode::ChooseMode);

    let handle_primary = move |_| {
        if device_name().trim().is_empty() {
            return;
        }
        let device = device_name().trim().to_string();
        let mut vault_status = vault_status.clone();
        let mut reload_trigger = reload_trigger.clone();

        vault_status.set(VaultStatus::SettingUpPrimary {
            device_name: device.clone(),
        });

        spawn(async move {
            let home_dir = match dirs::home_dir() {
                Some(dir) => dir,
                None => {
                    vault_status.set(VaultStatus::Error("Could not find home directory".to_string()));
                    return;
                }
            };

            let vault_path = home_dir.join("footnotes");

            match crate::core::init::init(
                Some(vault_path.clone()),
                Some("me"),
                Some(&device)
            ).await {
                Ok(_) => {
                    // Trigger reload by incrementing reload_trigger
                    vault_status.set(VaultStatus::Initializing);
                    reload_trigger.set(reload_trigger() + 1);
                },
                Err(e) => {
                    vault_status.set(VaultStatus::Error(format!("Failed to initialize: {}", e)));
                }
            }
        });
    };

    let handle_secondary = move |_| {
        if device_name().trim().is_empty() || connect_url().trim().is_empty() {
            return;
        }
        let device = device_name().trim().to_string();
        let url = connect_url().trim().to_string();
        let mut vault_status = vault_status.clone();
        let mut reload_trigger = reload_trigger.clone();

        vault_status.set(VaultStatus::SettingUpSecondary {
            device_name: device.clone(),
            connect_url: url.clone(),
        });

        spawn(async move {
            let home_dir = match dirs::home_dir() {
                Some(dir) => dir,
                None => {
                    vault_status.set(VaultStatus::Error("Could not find home directory".to_string()));
                    return;
                }
            };

            let vault_path = home_dir.join("footnotes");

            // Create the footnotes directory first
            if let Err(e) = std::fs::create_dir_all(&vault_path) {
                vault_status.set(VaultStatus::Error(format!("Failed to create directory: {}", e)));
                return;
            }

            // Change to that directory before running create_remote
            if let Err(e) = std::env::set_current_dir(&vault_path) {
                vault_status.set(VaultStatus::Error(format!("Failed to set working directory: {}", e)));
                return;
            }

            match crate::core::device::create_remote(&url, &device).await {
                Ok(_) => {
                    // Trigger reload by incrementing reload_trigger
                    vault_status.set(VaultStatus::Initializing);
                    reload_trigger.set(reload_trigger() + 1);
                },
                Err(e) => {
                    vault_status.set(VaultStatus::Error(format!("Failed to connect: {}", e)));
                }
            }
        });
    };

    rsx! {
        div { class: "flex items-center justify-center h-full",
            div { class: "max-w-md w-full p-8 bg-white rounded-lg shadow-lg",
                h1 { class: "text-2xl font-bold mb-6 text-center", "Welcome to Footnote" }

                div { class: "mb-6",
                    label { class: "block text-sm font-medium text-gray-700 mb-2", "Device Name" }
                    input {
                        r#type: "text",
                        class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                        placeholder: "e.g., laptop, desktop, phone",
                        value: "{device_name}",
                        oninput: move |evt| device_name.set(evt.value()),
                    }
                }

                if setup_mode() == SetupMode::ChooseMode {
                    div { class: "space-y-3",
                        button {
                            class: "w-full px-4 py-3 bg-blue-600 text-white rounded-md hover:bg-blue-700 font-medium disabled:bg-gray-300 disabled:cursor-not-allowed",
                            disabled: device_name().trim().is_empty(),
                            onclick: handle_primary,
                            "Primary Device"
                        }
                        button {
                            class: "w-full px-4 py-3 bg-white text-gray-700 border border-gray-300 rounded-md hover:bg-gray-50 font-medium disabled:bg-gray-100 disabled:cursor-not-allowed",
                            disabled: device_name().trim().is_empty(),
                            onclick: move |_| setup_mode.set(SetupMode::ConnectToExisting),
                            "Connect to Existing"
                        }
                    }
                } else {
                    div { class: "space-y-4",
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-2", "Connect URL" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono text-sm",
                                placeholder: "iroh://...",
                                value: "{connect_url}",
                                oninput: move |evt| connect_url.set(evt.value()),
                            }
                        }
                        div { class: "flex gap-2",
                            button {
                                class: "flex-1 px-4 py-2 bg-white text-gray-700 border border-gray-300 rounded-md hover:bg-gray-50",
                                onclick: move |_| setup_mode.set(SetupMode::ChooseMode),
                                "Back"
                            }
                            button {
                                class: "flex-1 px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed",
                                disabled: connect_url().trim().is_empty(),
                                onclick: handle_secondary,
                                "Connect"
                            }
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
    let mut all_files = use_signal(|| Vec::<String>::new());
    let mut picker_input = use_signal(|| String::new());
    let mut show_dropdown = use_signal(|| false);

    // Initialize edited_content when file loads
    use_effect(move || {
        if let Some(ref file_data) = *open_file.read() {
            edited_content.set(file_data.content.clone());
        }
    });

    // Scan notes directory for all markdown files on mount
    use_effect(move || {
        spawn(async move {
            let vault_path = match dirs::home_dir() {
                Some(dir) => dir.join("footnotes").join("notes"),
                None => return,
            };

            if let Ok(entries) = std::fs::read_dir(vault_path) {
                let mut files = Vec::new();
                for entry in entries.flatten() {
                    if let Ok(file_name) = entry.file_name().into_string() {
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
                            note.frontmatter.modified = chrono::Utc::now();

                            match crate::core::note::serialize_note(&note) {
                                Ok(serialized) => {
                                    match std::fs::write(&path, serialized) {
                                        Ok(_) => {
                                            save_status.set("Saved!".to_string());

                                            if let Some(file) = open_file.write().as_mut() {
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
        let share_with = file_data.share_with.clone();

        // Compute fuzzy-matched files
        let matcher = SkimMatcherV2::default();
        let mut filtered_files: Vec<(String, i64)> = all_files()
            .iter()
            .filter_map(|file| {
                if picker_input().is_empty() {
                    Some((file.clone(), 0))
                } else {
                    matcher.fuzzy_match(file, &picker_input()).map(|score| (file.clone(), score))
                }
            })
            .collect();
        filtered_files.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by score descending
        let filtered_files: Vec<String> = filtered_files.into_iter().map(|(f, _)| f).take(10).collect();

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
                                                let vault_path = match dirs::home_dir() {
                                                    Some(dir) => dir.join("footnotes").join("notes"),
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
                                                                filename: file_name,
                                                                content: note.content,
                                                                share_with: note.frontmatter.share_with,
                                                            }));
                                                            editor_mode.set(EditorMode::View);
                                                            picker_input.set(String::new());
                                                            show_dropdown.set(false);
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
                                    let vault_path = match dirs::home_dir() {
                                        Some(dir) => dir.join("footnotes"),
                                        None => return,
                                    };

                                    let file_path = vault_path.join("notes").join(&href);
                                    let mut open_file = open_file.clone();
                                    let mut editor_mode = editor_mode.clone();

                                    spawn(async move {
                                        // Check if file exists
                                        if !file_path.exists() {
                                            // Create new file with basic frontmatter
                                            let uuid = uuid::Uuid::new_v4();
                                            let new_content = format!(
                                                r#"---
uuid: {}
modified: {}
share_with: []
---

# {}
"#,
                                                uuid,
                                                chrono::Utc::now().to_rfc3339(),
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

#[derive(Clone, PartialEq)]
enum DeviceAddState {
    Idle,
    WaitingForConnection { join_url: String },
    Success,
    Error(String),
}

#[component]
fn ContactsScreen() -> Element {
    let mut self_contact = use_signal(|| None::<crate::core::crypto::ContactRecord>);
    let mut trusted_contacts = use_signal(|| Vec::<(String, crate::core::crypto::ContactRecord)>::new());
    let mut device_add_state = use_signal(|| DeviceAddState::Idle);
    let reload_trigger = use_signal(|| 0);

    // Load contacts on mount and when reload_trigger changes
    use_effect(move || {
        let _ = reload_trigger(); // Subscribe to changes
        spawn(async move {
            let vault_path = match dirs::home_dir() {
                Some(dir) => dir.join("footnotes").join(".footnotes"),
                None => return,
            };

            // Load self contact
            let self_path = vault_path.join("contact.json");
            if let Ok(content) = std::fs::read_to_string(&self_path) {
                if let Ok(contact) = serde_json::from_str::<crate::core::crypto::ContactRecord>(&content) {
                    self_contact.set(Some(contact));
                }
            }

            // Load trusted contacts
            let contacts_dir = vault_path.join("contacts");
            if let Ok(entries) = std::fs::read_dir(contacts_dir) {
                let mut contacts = Vec::new();
                for entry in entries.flatten() {
                    if let Ok(file_name) = entry.file_name().into_string() {
                        if file_name.ends_with(".json") {
                            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                                if let Ok(contact) = serde_json::from_str::<crate::core::crypto::ContactRecord>(&content) {
                                    let petname = file_name.trim_end_matches(".json").to_string();
                                    contacts.push((petname, contact));
                                }
                            }
                        }
                    }
                }
                contacts.sort_by(|a, b| a.0.cmp(&b.0));
                trusted_contacts.set(contacts);
            }
        });
    });

    rsx! {
        div { class: "max-w-4xl mx-auto p-6",
            // Me section
            div { class: "mb-8",
                div { class: "flex items-center justify-between mb-4",
                    h2 { class: "text-xl font-bold", "Me" }
                    if matches!(device_add_state(), DeviceAddState::Idle) {
                        button {
                            class: "px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700",
                            onclick: move |_| {
                                let mut device_add_state = device_add_state.clone();
                                let mut reload_trigger = reload_trigger.clone();

                                spawn(async move {
                                    match crate::core::device::create_primary().await {
                                        Ok(_) => {
                                            device_add_state.set(DeviceAddState::Success);
                                            // Reload contacts
                                            reload_trigger.set(reload_trigger() + 1);
                                        }
                                        Err(e) => {
                                            device_add_state.set(DeviceAddState::Error(e.to_string()));
                                        }
                                    }
                                });

                                // We can't easily extract the URL from create_primary
                                // For now, set a placeholder state
                                device_add_state.set(DeviceAddState::WaitingForConnection {
                                    join_url: "Generating...".to_string()
                                });
                            },
                            "Add Device"
                        }
                    }
                }

                if let Some(ref contact) = *self_contact.read() {
                    div { class: "bg-blue-50 border border-blue-200 rounded-md p-4",
                        div { class: "font-semibold", "{contact.username}" }
                        div { class: "text-sm text-gray-600 mt-1",
                            "{contact.devices.len()} device(s)"
                        }
                    }
                } else {
                    div { class: "text-gray-500 italic", "Loading..." }
                }

                // Device pairing UI
                match device_add_state() {
                    DeviceAddState::WaitingForConnection { ref join_url } => rsx! {
                        div { class: "mt-4 p-4 bg-yellow-50 border border-yellow-200 rounded-md",
                            div { class: "font-semibold mb-2", "Waiting for device..." }
                            div { class: "text-sm mb-2", "Copy this URL to your new device:" }
                            div { class: "font-mono text-xs bg-white p-2 rounded border break-all",
                                "{join_url}"
                            }
                            div { class: "text-sm text-gray-600 mt-2 italic",
                                "Listening for connection... (check console for URL)"
                            }
                        }
                    },
                    DeviceAddState::Success => rsx! {
                        div { class: "mt-4 p-4 bg-green-50 border border-green-200 rounded-md",
                            div { class: "font-semibold", "✓ Device added successfully!" }
                            button {
                                class: "mt-2 text-sm text-blue-600 hover:underline",
                                onclick: move |_| device_add_state.set(DeviceAddState::Idle),
                                "Done"
                            }
                        }
                    },
                    DeviceAddState::Error(ref error) => rsx! {
                        div { class: "mt-4 p-4 bg-red-50 border border-red-200 rounded-md",
                            div { class: "font-semibold text-red-700", "Error" }
                            div { class: "text-sm mt-1", "{error}" }
                            button {
                                class: "mt-2 text-sm text-blue-600 hover:underline",
                                onclick: move |_| device_add_state.set(DeviceAddState::Idle),
                                "Try Again"
                            }
                        }
                    },
                    DeviceAddState::Idle => rsx! {},
                }
            }

            // Contacts section
            div {
                h2 { class: "text-xl font-bold mb-4", "Contacts" }
                if trusted_contacts().is_empty() {
                    div { class: "text-gray-500 italic", "No contacts yet" }
                } else {
                    div { class: "space-y-2",
                        for (petname, contact) in trusted_contacts().iter() {
                            div {
                                key: "{petname}",
                                class: "bg-white border border-gray-200 rounded-md p-4 hover:border-gray-300",
                                div { class: "font-semibold", "{petname}" }
                                div { class: "text-sm text-gray-600 mt-1",
                                    "username: {contact.username}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
