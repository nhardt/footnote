use crate::ui::markdown::SimpleMarkdown;
use dioxus::prelude::*;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::path::PathBuf;
use tracing;

#[derive(Clone, Copy)]
struct VaultContext {
    vault_path: Signal<Option<PathBuf>>,
}

impl VaultContext {
    fn new() -> Self {
        Self {
            vault_path: Signal::new(None),
        }
    }

    fn set_vault(&mut self, path: PathBuf) {
        self.vault_path.set(Some(path));
    }

    fn get_vault(&self) -> Option<PathBuf> {
        self.vault_path.cloned()
    }

    fn clear_vault(&mut self) {
        self.vault_path.set(None);
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Screen {
    Editor,
    Contacts,
    Sync,
}

#[derive(Clone, PartialEq)]
enum VaultStatus {
    VaultNeeded,
    BrowsingToCreate,
    BrowsingToOpen,
    BrowsingToJoin,
    Creating {
        vault_path: PathBuf,
    },
    Opening {
        vault_path: PathBuf,
    },
    Joining {
        vault_path: PathBuf,
        device_name: String,
        connect_url: String,
    },
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
    let vault_status = use_signal(|| VaultStatus::VaultNeeded);
    let mut open_file = use_signal(|| None::<OpenFile>);

    use_context_provider(|| VaultContext::new());
    let vault_ctx = use_context::<VaultContext>();

    // Load config on app startup (once)
    let mut config_loaded = use_signal(|| false);
    use_effect(move || {
        if !config_loaded() {
            config_loaded.set(true);

            let mut vault_ctx = vault_ctx.clone();
            let mut open_file = open_file.clone();

            spawn(async move {
                if let Some(config) = crate::ui::config::AppConfig::load() {
                    // Validate vault exists
                    if !config.validate_vault() {
                        tracing::info!("Config vault path invalid, ignoring config");
                        return;
                    }

                    // Set vault context
                    vault_ctx.set_vault(config.last_vault_path.clone());

                    // Try to load the last file if it exists
                    if let Some(filename) = config.last_file {
                        let file_path = config.last_vault_path.join(&filename);
                        if file_path.exists() {
                            if let Ok(note) = crate::core::note::parse_note(&file_path) {
                                open_file.set(Some(OpenFile {
                                    path: file_path,
                                    filename: filename.clone(),
                                    content: note.content,
                                    share_with: note.frontmatter.share_with,
                                }));
                            }
                        }
                        // If file doesn't exist, fall through to home file loader
                    }
                }
            });
        }
    });

    // Load home file when vault context changes
    use_effect(move || {
        // Don't override if file already loaded from config
        if open_file.read().is_some() {
            return;
        }

        if let Some(vault_path) = vault_ctx.get_vault() {
            let vault_path_for_spawn = vault_path.clone();
            spawn(async move {
                // Get the local device name to load correct home file
                let device_name =
                    match crate::core::device::get_local_device_name(&vault_path_for_spawn) {
                        Ok(name) => name,
                        Err(_) => return,
                    };

                // Load device-specific home file from vault root
                let home_filename = format!("home-{}.md", device_name);
                let home_path = vault_path_for_spawn.join(&home_filename);

                if let Ok(note) = crate::core::note::parse_note(&home_path) {
                    open_file.set(Some(OpenFile {
                        path: home_path,
                        filename: home_filename,
                        content: note.content,
                        share_with: note.frontmatter.share_with,
                    }));
                }
            });
        }
    });

    rsx! {
        document::Stylesheet { href: asset!("/assets/tailwind.css") }

        div { class: "h-screen flex flex-col bg-gray-50",
            if vault_ctx.get_vault().is_some() {
                // Navigation bar
                nav { class: "bg-white border-b border-gray-200 px-4 py-3",
                    div { class: "flex justify-between items-center",
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
                            button {
                                class: if current_screen() == Screen::Sync { "px-4 py-2 font-medium text-blue-600 border-b-2 border-blue-600" } else { "px-4 py-2 font-medium text-gray-600 hover:text-gray-900" },
                                onclick: move |_| current_screen.set(Screen::Sync),
                                "Sync"
                            }
                        }
                        button {
                            class: "px-4 py-2 text-sm text-gray-600 hover:text-gray-900 border border-gray-300 rounded-md hover:bg-gray-50",
                            onclick: move |_| {
                                let mut vault_ctx = vault_ctx.clone();
                                let mut open_file = open_file.clone();
                                vault_ctx.clear_vault();
                                open_file.set(None);

                                // Clear config when switching vaults
                                spawn(async move {
                                    if let Err(e) = crate::ui::config::AppConfig::delete() {
                                        tracing::warn!("Failed to delete config: {}", e);
                                    }
                                });
                            },
                            "Switch Vault"
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
                        Screen::Sync => rsx! {
                            SyncScreen {}
                        },
                    }
                }
            } else {
                match vault_status() {
                    VaultStatus::VaultNeeded => rsx! {
                        VaultNeededScreen { vault_status }
                    },
                    VaultStatus::BrowsingToCreate => rsx! {
                        DirectoryBrowserScreen { vault_status, action: "Create" }
                    },
                    VaultStatus::BrowsingToOpen => rsx! {
                        DirectoryBrowserScreen { vault_status, action: "Open" }
                    },
                    VaultStatus::BrowsingToJoin => rsx! {
                        DirectoryBrowserScreen { vault_status, action: "Join" }
                    },
                    VaultStatus::Creating { ref vault_path } => rsx! {
                        CreateVaultScreen { vault_status, vault_path: vault_path.clone() }
                    },
                    VaultStatus::Opening { ref vault_path } => rsx! {
                        OpenVaultScreen { vault_status, vault_path: vault_path.clone() }
                    },
                    VaultStatus::Joining { ref vault_path, ref device_name, ref connect_url } => rsx! {
                        JoinVaultScreen { vault_status, vault_path: vault_path.clone(), device_name: device_name.clone(), connect_url: connect_url.clone() }
                    },
                    VaultStatus::Error(ref error) => rsx! {
                        div { class: "flex items-center justify-center h-full",
                            div { class: "text-center max-w-md",
                                div { class: "text-lg font-medium text-red-600", "Error" }
                                div { class: "text-sm text-gray-700 mt-2", "{error}" }
                            }
                        }
                    },
                }
            }
        }
    }
}

#[component]
fn DirectoryBrowserScreen(mut vault_status: Signal<VaultStatus>, action: &'static str) -> Element {
    let mut current_path = use_signal(|| match crate::platform::get_app_dir() {
        Ok(path) => {
            tracing::info!("Directory browser starting at: {}", path.display());
            path
        }
        Err(e) => {
            tracing::error!("Failed to get app directory: {}", e);
            PathBuf::from("/")
        }
    });
    let mut folders = use_signal(|| Vec::<PathBuf>::new());
    let mut new_folder_name = use_signal(|| String::new());
    let mut show_new_folder_input = use_signal(|| false);
    let mut has_footnotes_dir = use_signal(|| false);

    // Load folders whenever current_path changes
    use_effect(move || {
        let path = current_path();
        spawn(async move {
            let mut folder_list = Vec::new();
            if let Ok(entries) = std::fs::read_dir(&path) {
                for entry in entries.flatten() {
                    if entry.file_name().to_string_lossy().starts_with('.') {
                        continue;
                    }

                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_dir() {
                            folder_list.push(entry.path());
                        }
                    }
                }
            }
            folder_list.sort();
            folders.set(folder_list);

            // Check if .footnotes directory exists (for "Open" action)
            let footnotes_path = path.join(".footnotes");
            has_footnotes_dir.set(footnotes_path.exists() && footnotes_path.is_dir());
        });
    });

    let handle_go_up = move |_| {
        if let Some(parent) = current_path().parent() {
            current_path.set(parent.to_path_buf());
        }
    };

    let handle_select_here = move |_| {
        let path = current_path();
        if action == "Create" {
            vault_status.set(VaultStatus::Creating { vault_path: path });
        } else if action == "Join" {
            vault_status.set(VaultStatus::Joining {
                vault_path: path,
                device_name: String::new(),
                connect_url: String::new(),
            });
        } else {
            vault_status.set(VaultStatus::Opening { vault_path: path });
        }
    };

    let handle_cancel = move |_| {
        vault_status.set(VaultStatus::VaultNeeded);
    };

    let handle_create_folder = move |_| {
        if new_folder_name().trim().is_empty() {
            return;
        }

        let folder_name = new_folder_name().trim().to_string();
        let new_path = current_path().join(&folder_name);

        tracing::info!("Attempting to create directory: {}", new_path.display());

        if let Err(e) = std::fs::create_dir(&new_path) {
            tracing::error!(
                "Failed to create directory {}: {} (kind: {:?}, errno: {:?})",
                new_path.display(),
                e,
                e.kind(),
                e.raw_os_error()
            );
            // TODO: Show error to user
            return;
        }

        tracing::info!("Successfully created directory: {}", new_path.display());

        // Navigate into the newly created folder
        current_path.set(new_path);
        new_folder_name.set(String::new());
        show_new_folder_input.set(false);
    };

    let handle_toggle_new_folder = move |_| {
        show_new_folder_input.set(!show_new_folder_input());
        if show_new_folder_input() {
            new_folder_name.set(String::new());
        }
    };

    rsx! {
        div { class: "flex items-center justify-center h-full p-4",
            div { class: "max-w-2xl w-full bg-white rounded-lg shadow-lg",
                div { class: "p-6 border-b border-gray-200",
                    h1 { class: "text-2xl font-bold text-center", "Select Directory" }
                }

                div { class: "p-6",
                    div { class: "mb-4",
                        label { class: "block text-sm font-medium text-gray-700 mb-2", "Current Path" }
                        div { class: "flex gap-2",
                            div { class: "flex-1 px-3 py-2 border border-gray-300 rounded-md bg-gray-50 text-gray-900 font-mono text-sm break-all",
                                "{current_path().display()}"
                            }
                            button {
                                class: "px-3 py-2 bg-white border border-gray-300 rounded-md hover:bg-gray-50",
                                onclick: handle_go_up,
                                "↑ Up"
                            }
                            button {
                                class: "px-3 py-2 bg-white border border-gray-300 rounded-md hover:bg-gray-50",
                                onclick: handle_toggle_new_folder,
                                "+ Folder"
                            }
                        }
                    }

                    if show_new_folder_input() {
                        div { class: "mb-4",
                            label { class: "block text-sm font-medium text-gray-700 mb-2", "New Folder Name" }
                            div { class: "flex gap-2",
                                input {
                                    r#type: "text",
                                    class: "flex-1 px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                    placeholder: "folder-name",
                                    value: "{new_folder_name}",
                                    oninput: move |evt| new_folder_name.set(evt.value()),
                                    autofocus: true,
                                }
                                button {
                                    class: "px-3 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed",
                                    disabled: new_folder_name().trim().is_empty(),
                                    onclick: handle_create_folder,
                                    "Create"
                                }
                            }
                        }
                    }

                    div { class: "mb-4 max-h-96 overflow-y-auto border border-gray-200 rounded-md",
                        if folders().is_empty() {
                            div { class: "p-4 text-center text-gray-500", "No subdirectories" }
                        } else {
                            for folder in folders() {
                                {
                                    let folder_name = folder
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("?");
                                    let folder_path = folder.clone();
                                    rsx! {
                                        div {
                                            key: "{folder.display()}",
                                            class: "px-4 py-2 hover:bg-blue-50 cursor-pointer border-b border-gray-100 last:border-b-0",
                                            onclick: move |_| current_path.set(folder_path.clone()),
                                            "📁 {folder_name}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "p-6 border-t border-gray-200 flex gap-2",
                    button {
                        class: "flex-1 px-4 py-2 bg-white text-gray-700 border border-gray-300 rounded-md hover:bg-gray-50",
                        onclick: handle_cancel,
                        "Cancel"
                    }
                    button {
                        class: if action == "Open" && !has_footnotes_dir() {
                            "flex-1 px-4 py-2 bg-gray-300 text-gray-500 rounded-md cursor-not-allowed"
                        } else {
                            "flex-1 px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700"
                        },
                        disabled: action == "Open" && !has_footnotes_dir(),
                        onclick: handle_select_here,
                        "{action} Here"
                    }
                }
            }
        }
    }
}

#[component]
fn OpenVaultScreen(mut vault_status: Signal<VaultStatus>, vault_path: PathBuf) -> Element {
    let vault_path_display = vault_path.display().to_string();
    let vault_ctx = use_context::<VaultContext>();

    // Auto-open vault on mount
    use_effect(move || {
        let vault_path = vault_path.clone();
        let mut vault_status = vault_status.clone();
        let mut vault_ctx = vault_ctx.clone();

        spawn(async move {
            // Validate this is a vault directory
            let footnotes_dir = vault_path.join(".footnotes");
            if !footnotes_dir.exists() {
                vault_status.set(VaultStatus::Error(format!(
                    "Not a valid vault: {} (missing .footnotes directory)",
                    vault_path.display()
                )));
                return;
            }

            // Set the vault as working directory
            if let Err(e) = std::env::set_current_dir(&vault_path) {
                vault_status.set(VaultStatus::Error(format!(
                    "Failed to set working directory: {}",
                    e
                )));
                return;
            }

            // Get the local device name
            let device_name = match crate::core::device::get_local_device_name(&vault_path) {
                Ok(name) => name,
                Err(e) => {
                    vault_status.set(VaultStatus::Error(format!(
                        "Failed to get device name: {}",
                        e
                    )));
                    return;
                }
            };

            // Check for device-specific home file, create if it doesn't exist
            let home_filename = format!("home-{}.md", device_name);
            let home_path = vault_path.join(&home_filename);

            if !home_path.exists() {
                // Create device-specific home file
                let uuid = uuid::Uuid::new_v4();
                let vector_time = crate::core::note::VectorTime::default();
                let home_content = format!(
                    r#"---
uuid: {}
modified: {}
share_with: []
---

# Home ({})

Welcome to footnote! This is your home note.
"#,
                    uuid,
                    vector_time.as_i64(),
                    device_name
                );

                if let Err(e) = std::fs::write(&home_path, home_content) {
                    vault_status.set(VaultStatus::Error(format!(
                        "Failed to create home file: {}",
                        e
                    )));
                    return;
                }
            }

            vault_ctx.set_vault(vault_path.clone());

            // Save vault to config
            spawn(async move {
                let config = crate::ui::config::AppConfig {
                    last_vault_path: vault_path,
                    last_file: None,
                };
                if let Err(e) = config.save() {
                    tracing::warn!("Failed to save config: {}", e);
                }
            });

            vault_status.set(VaultStatus::VaultNeeded);
        });
    });

    rsx! {
        div { class: "flex items-center justify-center h-full",
            div { class: "text-center",
                div { class: "text-lg font-medium text-gray-700", "Opening vault..." }
                div { class: "text-sm text-gray-500 mt-2", "{vault_path_display}" }
            }
        }
    }
}

#[component]
fn JoinVaultScreen(
    mut vault_status: Signal<VaultStatus>,
    vault_path: PathBuf,
    device_name: String,
    connect_url: String,
) -> Element {
    let mut device_name_input = use_signal(|| device_name);
    let mut connect_url_input = use_signal(|| connect_url);
    let vault_path_display = vault_path.display().to_string();
    let vault_ctx = use_context::<VaultContext>();

    let handle_join = move |_| {
        if device_name_input().trim().is_empty() || connect_url_input().trim().is_empty() {
            return;
        }

        let device = device_name_input().trim().to_string();
        let url = connect_url_input().trim().to_string();
        let vault_path = vault_path.clone();
        let mut vault_status = vault_status.clone();
        let mut vault_ctx = vault_ctx.clone();

        spawn(async move {
            if let Err(e) = std::env::set_current_dir(&vault_path) {
                vault_status.set(VaultStatus::Error(format!(
                    "Failed to set working directory: {}",
                    e
                )));
                return;
            }

            match crate::core::device::create_remote(&vault_path, &url, &device).await {
                Ok(_) => {
                    vault_ctx.set_vault(vault_path);
                    vault_status.set(VaultStatus::VaultNeeded);
                }
                Err(e) => {
                    vault_status.set(VaultStatus::Error(format!("Failed to join vault: {}", e)));
                }
            }
        });
    };

    let handle_cancel = move |_| {
        vault_status.set(VaultStatus::VaultNeeded);
    };

    rsx! {
        div { class: "flex items-center justify-center h-full",
            div { class: "max-w-md w-full p-8 bg-white rounded-lg shadow-lg",
                h1 { class: "text-2xl font-bold mb-6 text-center", "Join Vault" }

                div { class: "mb-4",
                    label { class: "block text-sm font-medium text-gray-700 mb-2", "Vault Location" }
                    div { class: "px-3 py-2 border border-gray-300 rounded-md bg-gray-50 text-gray-900 font-mono text-sm break-all",
                        "{vault_path_display}"
                    }
                }

                div { class: "mb-4",
                    label { class: "block text-sm font-medium text-gray-700 mb-2", "Device Name" }
                    input {
                        r#type: "text",
                        class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                        placeholder: "e.g., laptop, desktop, phone",
                        value: "{device_name_input}",
                        oninput: move |evt| device_name_input.set(evt.value()),
                        autofocus: true,
                    }
                }

                div { class: "mb-6",
                    label { class: "block text-sm font-medium text-gray-700 mb-2", "Connection URL" }
                    input {
                        r#type: "text",
                        class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono text-sm",
                        placeholder: "iroh://...",
                        value: "{connect_url_input}",
                        oninput: move |evt| connect_url_input.set(evt.value()),
                    }
                }

                div { class: "flex gap-2",
                    button {
                        class: "flex-1 px-4 py-2 bg-white text-gray-700 border border-gray-300 rounded-md hover:bg-gray-50",
                        onclick: handle_cancel,
                        "Cancel"
                    }
                    button {
                        class: "flex-1 px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed",
                        disabled: device_name_input().trim().is_empty() || connect_url_input().trim().is_empty(),
                        onclick: handle_join,
                        "Join Vault"
                    }
                }
            }
        }
    }
}

#[component]
fn CreateVaultScreen(mut vault_status: Signal<VaultStatus>, vault_path: PathBuf) -> Element {
    let mut device_name = use_signal(|| String::new());
    let vault_path_display = vault_path.display().to_string();
    let vault_ctx = use_context::<VaultContext>();

    let handle_create = move |_| {
        if device_name().trim().is_empty() {
            return;
        }

        let device = device_name().trim().to_string();
        let vault_path = vault_path.clone();
        let mut vault_status = vault_status.clone();
        let mut vault_ctx = vault_ctx.clone();

        spawn(async move {
            match crate::core::init::init(Some(vault_path.clone()), Some("me"), Some(&device)).await
            {
                Ok(_) => {
                    // Set the vault as working directory
                    if let Err(e) = std::env::set_current_dir(&vault_path) {
                        vault_status.set(VaultStatus::Error(format!(
                            "Failed to set working directory: {}",
                            e
                        )));
                        return;
                    }

                    vault_ctx.set_vault(vault_path);
                    vault_status.set(VaultStatus::VaultNeeded);
                }
                Err(e) => {
                    vault_status.set(VaultStatus::Error(format!(
                        "Failed to initialize vault: {}",
                        e
                    )));
                }
            }
        });
    };

    let handle_cancel = move |_| {
        vault_status.set(VaultStatus::VaultNeeded);
    };

    rsx! {
        div { class: "flex items-center justify-center h-full",
            div { class: "max-w-md w-full p-8 bg-white rounded-lg shadow-lg",
                h1 { class: "text-2xl font-bold mb-6 text-center", "Create New Vault" }

                div { class: "mb-4",
                    label { class: "block text-sm font-medium text-gray-700 mb-2", "Vault Location" }
                    div { class: "px-3 py-2 border border-gray-300 rounded-md bg-gray-50 text-gray-900 font-mono text-sm break-all",
                        "{vault_path_display}"
                    }
                }

                div { class: "mb-6",
                    label { class: "block text-sm font-medium text-gray-700 mb-2", "Device Name" }
                    input {
                        r#type: "text",
                        class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                        placeholder: "e.g., laptop, desktop, phone",
                        value: "{device_name}",
                        oninput: move |evt| device_name.set(evt.value()),
                        autofocus: true,
                    }
                }

                div { class: "flex gap-2",
                    button {
                        class: "flex-1 px-4 py-2 bg-white text-gray-700 border border-gray-300 rounded-md hover:bg-gray-50",
                        onclick: handle_cancel,
                        "Cancel"
                    }
                    button {
                        class: "flex-1 px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed",
                        disabled: device_name().trim().is_empty(),
                        onclick: handle_create,
                        "Create Vault"
                    }
                }
            }
        }
    }
}

#[component]
fn VaultNeededScreen(mut vault_status: Signal<VaultStatus>) -> Element {
    let handle_create = move |_| {
        vault_status.set(VaultStatus::BrowsingToCreate);
    };

    let handle_join = move |_| {
        vault_status.set(VaultStatus::BrowsingToJoin);
    };

    let handle_open = move |_| {
        vault_status.set(VaultStatus::BrowsingToOpen);
    };

    rsx! {
        div { class: "flex items-center justify-center h-full",
            div { class: "max-w-md w-full p-8 bg-white rounded-lg shadow-lg",
                h1 { class: "text-2xl font-bold mb-6 text-center", "Welcome to Footnote" }

                div { class: "space-y-3",
                    button {
                        class: "w-full px-4 py-3 bg-blue-600 text-white rounded-md hover:bg-blue-700 font-medium",
                        onclick: handle_create,
                        "Create"
                    }
                    button {
                        class: "w-full px-4 py-3 bg-white text-gray-700 border border-gray-300 rounded-md hover:bg-gray-50 font-medium",
                        onclick: handle_join,
                        "Join"
                    }
                    button {
                        class: "w-full px-4 py-3 bg-white text-gray-700 border border-gray-300 rounded-md hover:bg-gray-50 font-medium",
                        onclick: handle_open,
                        "Open"
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
    let vault_ctx = use_context::<VaultContext>();

    // Initialize edited_content when file loads
    use_effect(move || {
        if let Some(ref file_data) = *open_file.read() {
            edited_content.set(file_data.content.clone());
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
    Listening { join_url: String },
    Connecting,
    ReceivedRequest { device_name: String },
    Verifying,
    Success { device_name: String },
    Error(String),
}

#[component]
fn ContactsScreen() -> Element {
    let mut self_contact = use_signal(|| None::<crate::core::crypto::ContactRecord>);
    let mut trusted_contacts =
        use_signal(|| Vec::<(String, crate::core::crypto::ContactRecord)>::new());
    let mut device_add_state = use_signal(|| DeviceAddState::Idle);
    let reload_trigger = use_signal(|| 0);
    let vault_ctx = use_context::<VaultContext>();

    // Load contacts on mount and when reload_trigger changes
    use_effect(move || {
        let _ = reload_trigger(); // Subscribe to changes
        let vault_ctx = vault_ctx.clone();
        spawn(async move {
            let vault_path = match vault_ctx.get_vault() {
                Some(path) => path.join(".footnotes"),
                None => return,
            };

            // Load self contact
            let self_path = vault_path.join("contact.json");
            if let Ok(content) = std::fs::read_to_string(&self_path) {
                if let Ok(contact) =
                    serde_json::from_str::<crate::core::crypto::ContactRecord>(&content)
                {
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
                                if let Ok(contact) = serde_json::from_str::<
                                    crate::core::crypto::ContactRecord,
                                >(&content)
                                {
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
                                let vault_ctx = vault_ctx.clone();

                                spawn(async move {
                                    // Get vault path from context
                                    let vault_path = match vault_ctx.get_vault() {
                                        Some(path) => path,
                                        None => {
                                            device_add_state.set(DeviceAddState::Error(
                                                "No vault path available".to_string()
                                            ));
                                            return;
                                        }
                                    };

                                    match crate::core::device::create_primary(&vault_path).await {
                                        Ok(mut rx) => {
                                            // Consume events from the channel
                                            while let Some(event) = rx.recv().await {
                                                match event {
                                                    crate::core::device::DeviceAuthEvent::Listening { join_url } => {
                                                        device_add_state.set(DeviceAddState::Listening { join_url });
                                                    }
                                                    crate::core::device::DeviceAuthEvent::Connecting => {
                                                        device_add_state.set(DeviceAddState::Connecting);
                                                    }
                                                    crate::core::device::DeviceAuthEvent::ReceivedRequest { device_name } => {
                                                        device_add_state.set(DeviceAddState::ReceivedRequest { device_name });
                                                    }
                                                    crate::core::device::DeviceAuthEvent::Verifying => {
                                                        device_add_state.set(DeviceAddState::Verifying);
                                                    }
                                                    crate::core::device::DeviceAuthEvent::Success { device_name } => {
                                                        device_add_state.set(DeviceAddState::Success { device_name });
                                                        // Reload contacts
                                                        reload_trigger.set(reload_trigger() + 1);
                                                        break;
                                                    }
                                                    crate::core::device::DeviceAuthEvent::Error(err) => {
                                                        device_add_state.set(DeviceAddState::Error(err));
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            device_add_state.set(DeviceAddState::Error(e.to_string()));
                                        }
                                    }
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
                    DeviceAddState::Listening { ref join_url } => rsx! {
                        div { class: "mt-4 p-4 bg-yellow-50 border border-yellow-200 rounded-md",
                            div { class: "font-semibold mb-2", "🔐 Waiting for device..." }
                            div { class: "text-sm mb-2", "Copy this URL to your new device:" }
                            div { class: "font-mono text-xs bg-white p-2 rounded border break-all",
                                "{join_url}"
                            }
                            div { class: "text-sm text-gray-600 mt-2 italic",
                                "Listening for connection..."
                            }
                        }
                    },
                    DeviceAddState::Connecting => rsx! {
                        div { class: "mt-4 p-4 bg-blue-50 border border-blue-200 rounded-md",
                            div { class: "font-semibold", "✓ Device connecting..." }
                        }
                    },
                    DeviceAddState::ReceivedRequest { ref device_name } => rsx! {
                        div { class: "mt-4 p-4 bg-blue-50 border border-blue-200 rounded-md",
                            div { class: "font-semibold", "✓ Received request from: {device_name}" }
                        }
                    },
                    DeviceAddState::Verifying => rsx! {
                        div { class: "mt-4 p-4 bg-blue-50 border border-blue-200 rounded-md",
                            div { class: "font-semibold", "✓ Verifying..." }
                        }
                    },
                    DeviceAddState::Success { ref device_name } => rsx! {
                        div { class: "mt-4 p-4 bg-green-50 border border-green-200 rounded-md",
                            div { class: "font-semibold", "✓ Device '{device_name}' added successfully!" }
                            button {
                                class: "mt-2 text-sm text-blue-600 hover:underline",
                                onclick: move |_| device_add_state.set(DeviceAddState::Idle),
                                "Done"
                            }
                        }
                    },
                    DeviceAddState::Error(ref error) => rsx! {
                        div { class: "mt-4 p-4 bg-red-50 border border-red-200 rounded-md",
                            div { class: "font-semibold text-red-700", "✗ Error" }
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

#[derive(Clone, PartialEq)]
enum ListenStatus {
    Idle,
    Listening { endpoint_id: String },
    Received { from: String, endpoint_id: String },
    Error(String),
}

#[derive(Clone, PartialEq)]
enum SyncStatus {
    Idle,
    Syncing { device_name: String },
    Success { device_name: String },
    Error { device_name: String, error: String },
}

#[component]
fn SyncScreen() -> Element {
    let mut self_contact = use_signal(|| None::<crate::core::crypto::ContactRecord>);
    let mut trusted_contacts =
        use_signal(|| Vec::<(String, crate::core::crypto::ContactRecord)>::new());
    let sync_status = use_signal(|| SyncStatus::Idle);
    let mut listen_status = use_signal(|| ListenStatus::Idle);
    let mut cancel_token = use_signal(|| None::<tokio_util::sync::CancellationToken>);
    let confirm_delete = use_signal(|| None::<String>);
    let reload_trigger = use_signal(|| 0);
    let vault_ctx = use_context::<VaultContext>();

    // Load contacts on mount and when reload_trigger changes
    use_effect(move || {
        let _ = reload_trigger(); // Subscribe to changes
        let vault_ctx = vault_ctx.clone();
        spawn(async move {
            let vault_path = match vault_ctx.get_vault() {
                Some(path) => path.join(".footnotes"),
                None => return,
            };

            // Load self contact
            let self_path = vault_path.join("contact.json");
            if let Ok(content) = std::fs::read_to_string(&self_path) {
                if let Ok(contact) =
                    serde_json::from_str::<crate::core::crypto::ContactRecord>(&content)
                {
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
                                if let Ok(contact) = serde_json::from_str::<
                                    crate::core::crypto::ContactRecord,
                                >(&content)
                                {
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
            h1 { class: "text-2xl font-bold mb-6", "Device Sync" }

            // Me section
            div { class: "mb-8",
                h2 { class: "text-xl font-bold mb-4", "My Devices" }
                if let Some(ref contact) = *self_contact.read() {
                    div { class: "space-y-2",
                        for device in contact.devices.iter() {
                            {
                                let device_name = device.device_name.clone();
                                let endpoint_id = device.iroh_endpoint_id.clone();
                                let is_current = vault_ctx.get_vault()
                                    .and_then(|vp| crate::core::device::get_local_device_name(&vp).ok())
                                    .map(|name| name == device_name)
                                    .unwrap_or(false);

                                rsx! {
                                    div {
                                        key: "{endpoint_id}",
                                        class: "bg-white border border-gray-200 rounded-md p-4",
                                        div { class: "flex items-center justify-between",
                                            div { class: "flex-1",
                                                div { class: "font-semibold",
                                                    "{device_name}"
                                                    if is_current {
                                                        span { class: "ml-2 text-xs text-green-600 font-normal", "(this device)" }
                                                    }
                                                }
                                                div { class: "text-sm text-gray-600 mt-1 font-mono text-xs truncate",
                                                    "ID: {endpoint_id}"
                                                }
                                                div { class: "text-sm text-gray-500 mt-1",
                                                    match sync_status() {
                                                        SyncStatus::Idle => "Ready to sync".to_string(),
                                                        SyncStatus::Syncing { device_name: ref syncing_device } if syncing_device == &device_name => "Syncing...".to_string(),
                                                        SyncStatus::Success { device_name: ref success_device } if success_device == &device_name => "Last sync: just now".to_string(),
                                                        SyncStatus::Error { device_name: ref error_device, .. } if error_device == &device_name => "Last sync: failed".to_string(),
                                                        _ => "—".to_string(),
                                                    }
                                                }
                                            }
                                            if !is_current {
                                                {
                                                    let device_name = device_name.clone();
                                                    let endpoint_id = endpoint_id.clone();
                                                    let vault_ctx = vault_ctx.clone();
                                                    let sync_status = sync_status.clone();
                                                    let confirm_delete = confirm_delete.clone();

                                                    rsx! {
                                                        button {
                                                            class: "px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed",
                                                            disabled: !matches!(sync_status(), SyncStatus::Idle),
                                                            onclick: {
                                                                let device_name = device_name.clone();
                                                                let endpoint_id = endpoint_id.clone();
                                                                let vault_ctx = vault_ctx.clone();
                                                                let sync_status = sync_status.clone();
                                                                move |_| {
                                                                    let device_name = device_name.clone();
                                                                    let endpoint_id = endpoint_id.clone();
                                                                    let vault_ctx = vault_ctx.clone();
                                                                    let mut sync_status = sync_status.clone();

                                                                    spawn(async move {
                                                                    sync_status.set(SyncStatus::Syncing { device_name: device_name.clone() });

                                                                    let vault_path = match vault_ctx.get_vault() {
                                                                        Some(path) => path,
                                                                        None => {
                                                                            sync_status.set(SyncStatus::Error {
                                                                                device_name: device_name.clone(),
                                                                                error: "No vault path".to_string(),
                                                                            });
                                                                            return;
                                                                        }
                                                                    };

                                                                    let notes_dir = vault_path.clone();
                                                                    let footnotes_dir = vault_path.join(".footnotes");
                                                                    let key_file = footnotes_dir.join("this_device");

                                                                    // Load local secret key
                                                                    let secret_key = match std::fs::read(&key_file) {
                                                                        Ok(key_bytes) => {
                                                                            let key_array: Result<[u8; 32], _> = key_bytes.try_into();
                                                                            match key_array {
                                                                                Ok(arr) => iroh::SecretKey::from_bytes(&arr),
                                                                                Err(_) => {
                                                                                    sync_status.set(SyncStatus::Error {
                                                                                        device_name: device_name.clone(),
                                                                                        error: "Invalid key length".to_string(),
                                                                                    });
                                                                                    return;
                                                                                }
                                                                            }
                                                                        }
                                                                        Err(e) => {
                                                                            sync_status.set(SyncStatus::Error {
                                                                                device_name: device_name.clone(),
                                                                                error: format!("Failed to read secret key: {}", e),
                                                                            });
                                                                            return;
                                                                        }
                                                                    };

                                                                    // Parse endpoint ID
                                                                    match endpoint_id.parse::<iroh::PublicKey>() {
                                                                        Ok(public_key) => {
                                                                            match crate::core::sync::push_to_device(&notes_dir, public_key, secret_key).await {
                                                                                Ok(_) => {
                                                                                    sync_status.set(SyncStatus::Success { device_name: device_name.clone() });
                                                                                }
                                                                                Err(e) => {
                                                                                    sync_status.set(SyncStatus::Error {
                                                                                        device_name: device_name.clone(),
                                                                                        error: e.to_string(),
                                                                                    });
                                                                                }
                                                                            }
                                                                        }
                                                                        Err(e) => {
                                                                            sync_status.set(SyncStatus::Error {
                                                                                device_name: device_name.clone(),
                                                                                error: format!("Invalid endpoint ID: {}", e),
                                                                            });
                                                                        }
                                                                    }
                                                                    });
                                                                }
                                                            },
                                                            "Sync"
                                                        }
                                                        button {
                                                            class: "px-4 py-2 ml-2 bg-red-600 text-white rounded-md hover:bg-red-700 disabled:bg-gray-300 disabled:cursor-not-allowed",
                                                            disabled: confirm_delete().is_some(),
                                                            onclick: {
                                                                let device_name = device_name.clone();
                                                                let mut confirm_delete = confirm_delete.clone();
                                                                move |_| {
                                                                    confirm_delete.set(Some(device_name.clone()));
                                                                }
                                                            },
                                                            "Delete"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        if let SyncStatus::Error { device_name: ref error_device, ref error } = sync_status() {
                                            if error_device == &device_name {
                                                div { class: "mt-2 p-2 bg-red-50 border border-red-200 rounded text-sm text-red-700",
                                                    "Error: {error}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    div { class: "text-gray-500 italic", "Loading..." }
                }
            }

            // Confirmation dialog for device deletion
            if let Some(device_to_delete) = confirm_delete().clone() {
                div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
                    div { class: "bg-white rounded-lg p-6 max-w-md",
                        h3 { class: "text-lg font-bold mb-4", "Delete Device" }
                        p { class: "mb-4", "Are you sure you want to delete device '{device_to_delete}'?" }
                        div { class: "flex gap-2 justify-end",
                            button {
                                class: "px-4 py-2 bg-gray-300 rounded-md hover:bg-gray-400",
                                onclick: {
                                    let mut confirm_delete = confirm_delete.clone();
                                    move |_| {
                                        confirm_delete.set(None);
                                    }
                                },
                                "Cancel"
                            }
                            button {
                                class: "px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-700",
                                onclick: {
                                    let device_name = device_to_delete.clone();
                                    let mut confirm_delete = confirm_delete.clone();
                                    let mut reload_trigger = reload_trigger.clone();
                                    let vault_ctx = vault_ctx.clone();
                                    move |_| {
                                        let device_name = device_name.clone();
                                        let vault_ctx = vault_ctx.clone();
                                        spawn(async move {
                                            let vault_path = match vault_ctx.get_vault() {
                                                Some(path) => path,
                                                None => {
                                                    tracing::error!("No vault path available for delete");
                                                    confirm_delete.set(None);
                                                    return;
                                                }
                                            };

                                            match crate::core::device::delete_device(&vault_path, &device_name).await {
                                                Ok(_) => {
                                                    reload_trigger.set(reload_trigger() + 1);
                                                    confirm_delete.set(None);
                                                }
                                                Err(e) => {
                                                    tracing::error!("Failed to delete device: {}", e);
                                                    confirm_delete.set(None);
                                                }
                                            }
                                        });
                                    }
                                },
                                "Delete"
                            }
                        }
                    }
                }
            }

            // Receive Sync section
            div { class: "mb-8",
                h2 { class: "text-xl font-bold mb-4", "Receive Sync" }
                div { class: "bg-white border border-gray-200 rounded-md p-4",
                    div { class: "flex items-center justify-between",
                        div { class: "flex-1",
                            div { class: "font-semibold", "Accept sync from other devices" }
                            div { class: "text-sm text-gray-600 mt-1",
                                match listen_status() {
                                    ListenStatus::Idle => "Not listening".to_string(),
                                    ListenStatus::Listening { ref endpoint_id } => {
                                        format!("Listening on: {}...", &endpoint_id[..16.min(endpoint_id.len())])
                                    }
                                    ListenStatus::Received { ref from, .. } => {
                                        format!("Recently received sync from: {}", from)
                                    }
                                    ListenStatus::Error(ref e) => format!("Error: {}", e),
                                }
                            }
                        }
                        if matches!(listen_status(), ListenStatus::Listening { .. } | ListenStatus::Received { .. }) {
                            button {
                                class: "px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-700",
                                onclick: move |_| {
                                    // Stop listening
                                    if let Some(token) = cancel_token() {
                                        token.cancel();
                                        cancel_token.set(None);
                                        listen_status.set(ListenStatus::Idle);
                                    }
                                },
                                "Stop Listening"
                            }
                        } else {
                            button {
                                class: "px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 disabled:bg-gray-300",
                                disabled: matches!(listen_status(), ListenStatus::Error(_)),
                                onclick: move |_| {
                                    let mut listen_status = listen_status.clone();
                                    let mut cancel_token = cancel_token.clone();
                                    let vault_ctx = vault_ctx.clone();

                                    spawn(async move {
                                        let vault_path = match vault_ctx.get_vault() {
                                            Some(path) => path,
                                            None => {
                                                listen_status.set(ListenStatus::Error("No vault path available".to_string()));
                                                return;
                                            }
                                        };

                                        match crate::core::mirror::listen_background(&vault_path).await {
                                            Ok((mut rx, token)) => {
                                                cancel_token.set(Some(token));

                                                // Consume events from the channel
                                                while let Some(event) = rx.recv().await {
                                                    match event {
                                                        crate::core::mirror::ListenEvent::Started { endpoint_id } => {
                                                            listen_status.set(ListenStatus::Listening { endpoint_id: endpoint_id.clone() });
                                                        }
                                                        crate::core::mirror::ListenEvent::Received { from } => {
                                                            // Keep the endpoint_id when showing received status
                                                            if let ListenStatus::Listening { endpoint_id } = listen_status() {
                                                                listen_status.set(ListenStatus::Received {
                                                                    from: from.clone(),
                                                                    endpoint_id: endpoint_id.clone()
                                                                });

                                                                // Reset to listening after 3 seconds
                                                                let mut listen_status = listen_status.clone();
                                                                let endpoint_id_copy = endpoint_id.clone();
                                                                spawn(async move {
                                                                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                                                                    if matches!(listen_status(), ListenStatus::Received { .. }) {
                                                                        listen_status.set(ListenStatus::Listening { endpoint_id: endpoint_id_copy });
                                                                    }
                                                                });
                                                            }
                                                        }
                                                        crate::core::mirror::ListenEvent::Stopped => {
                                                            listen_status.set(ListenStatus::Idle);
                                                            cancel_token.set(None);
                                                            break;
                                                        }
                                                        crate::core::mirror::ListenEvent::Error(err) => {
                                                            listen_status.set(ListenStatus::Error(err));
                                                            cancel_token.set(None);
                                                            break;
                                                        }
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                listen_status.set(ListenStatus::Error(e.to_string()));
                                            }
                                        }
                                    });
                                },
                                "Start Listening"
                            }
                        }
                    }

                    // Show recent sync info
                    if let ListenStatus::Received { ref from, .. } = listen_status() {
                        div { class: "mt-2 p-2 bg-green-50 border border-green-200 rounded text-sm text-green-700",
                            "Received sync from {from}"
                        }
                    }

                    if let ListenStatus::Error(ref e) = listen_status() {
                        div { class: "mt-2 p-2 bg-red-50 border border-red-200 rounded text-sm text-red-700",
                            "Error: {e}"
                        }
                    }
                }
            }

            // Trusted contacts section
            div {
                h2 { class: "text-xl font-bold mb-4", "Trusted Contacts" }
                if trusted_contacts().is_empty() {
                    div { class: "text-gray-500 italic", "No trusted contacts yet" }
                } else {
                    div { class: "space-y-4",
                        for (petname, contact) in trusted_contacts().iter() {
                            div {
                                key: "{petname}",
                                class: "bg-white border border-gray-200 rounded-md p-4",
                                div { class: "font-semibold mb-2", "{petname} ({contact.username})" }
                                div { class: "space-y-2 ml-4",
                                    for device in contact.devices.iter() {
                                        {
                                            let device_name = device.device_name.clone();
                                            let endpoint_id = device.iroh_endpoint_id.clone();

                                            rsx! {
                                                div {
                                                    key: "{endpoint_id}",
                                                    class: "flex items-center justify-between border-l-2 border-gray-200 pl-3 py-2",
                                                    div { class: "flex-1",
                                                        div { class: "text-sm font-medium", "{device_name}" }
                                                        div { class: "text-xs text-gray-500 font-mono truncate", "ID: {endpoint_id}" }
                                                    }
                                                    div { class: "text-xs text-gray-400", "—" }
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
