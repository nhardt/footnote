use crate::ui::components::icons;
use crate::ui::context::VaultContext;
use crate::ui::screens::*;
use dioxus::prelude::*;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq)]
enum Screen {
    Editor,
    Contacts,
    Sync,
}

#[component]
pub fn App() -> Element {
    let mut current_screen = use_signal(|| Screen::Editor);
    let vault_status = use_signal(|| VaultStatus::VaultNeeded);
    let mut open_file = use_signal(|| None::<OpenFile>);
    let mut menu_open = use_signal(|| false);

    // Command palette state
    let mut palette_input = use_signal(|| String::new());
    let mut palette_open = use_signal(|| false);

    use_context_provider(|| VaultContext::new());
    let vault_ctx = use_context::<VaultContext>();

    use_load_last_session(vault_ctx, open_file);

    // Load home file when vault context changes
    use_effect(move || {
        if open_file.read().is_some() {
            tracing::debug!("skipping file load since open_file.read().is_some()");
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
                        footnotes: note.frontmatter.footnotes,
                    }));
                }
            });
        }
    });

    rsx! {
        document::Stylesheet { href: asset!("/assets/tailwind.css") }

        div { class: "h-screen flex flex-col bg-zinc-950",
            if vault_ctx.get_vault().is_some() {
                // Left slide-over menu
                if menu_open() {
                    div {
                        class: "fixed inset-0 z-50 overflow-hidden",

                        // Backdrop overlay
                        div {
                            class: "absolute inset-0 bg-zinc-950/75 transition-opacity duration-300",
                            onclick: move |_| menu_open.set(false),
                        }

                        // Panel container
                        div {
                            class: "absolute inset-0 pr-10 focus:outline-none sm:pr-16",

                            // Slide-over panel
                            div {
                                class: "relative mr-auto h-full w-full max-w-sm transform bg-zinc-900 shadow-xl transition-transform duration-300 ease-in-out",

                                // Close button
                                div {
                                    class: "absolute top-0 right-0 -mr-8 flex pt-4 pl-2 sm:-mr-10 sm:pl-4",
                                    button {
                                        r#type: "button",
                                        onclick: move |_| menu_open.set(false),
                                        class: "relative rounded-md text-zinc-400 hover:text-zinc-100 focus:outline-none focus:ring-2 focus:ring-slate-400",
                                        span { class: "absolute -inset-2.5" }
                                        span { class: "sr-only", "Close panel" }
                                        icons::CloseIcon {}
                                    }
                                }

                                // Panel content
                                div {
                                    class: "relative h-full overflow-y-auto p-6 flex flex-col",

                                    // Navigation menu
                                    nav { class: "flex flex-col gap-2",
                                        // Editor
                                        button {
                                            onclick: move |_| {
                                                current_screen.set(Screen::Editor);
                                                menu_open.set(false);
                                            },
                                            class: if current_screen() == Screen::Editor {
                                                "flex items-center gap-x-3 rounded-md bg-zinc-800 p-3 text-sm font-semibold text-zinc-100"
                                            } else {
                                                "flex items-center gap-x-3 rounded-md p-3 text-sm font-semibold text-zinc-200 hover:bg-zinc-800"
                                            },
                                            icons::EditIcon {}
                                            "Editor"
                                        }

                                        // Contacts
                                        button {
                                            onclick: move |_| {
                                                current_screen.set(Screen::Contacts);
                                                menu_open.set(false);
                                            },
                                            class: if current_screen() == Screen::Contacts {
                                                "flex items-center gap-x-3 rounded-md bg-zinc-800 p-3 text-sm font-semibold text-zinc-100"
                                            } else {
                                                "flex items-center gap-x-3 rounded-md p-3 text-sm font-semibold text-zinc-200 hover:bg-zinc-800"
                                            },
                                            icons::ContactsIcon {}
                                            "Contacts"
                                        }

                                        // Sync
                                        button {
                                            onclick: move |_| {
                                                current_screen.set(Screen::Sync);
                                                menu_open.set(false);
                                            },
                                            class: if current_screen() == Screen::Sync {
                                                "flex items-center gap-x-3 rounded-md bg-zinc-800 p-3 text-sm font-semibold text-zinc-100"
                                            } else {
                                                "flex items-center gap-x-3 rounded-md p-3 text-sm font-semibold text-zinc-200 hover:bg-zinc-800"
                                            },
                                            icons::SyncIcon {}
                                            "Sync"
                                        }
                                    }

                                    // Spacer to push Switch Vault to bottom
                                    div { class: "flex-1" }

                                    // Switch Vault button at bottom
                                    div { class: "border-t border-zinc-700 pt-4 mt-4",
                                        button {
                                            onclick: move |_| {
                                                let mut vault_ctx = vault_ctx.clone();
                                                let mut open_file = open_file.clone();
                                                vault_ctx.clear_vault();
                                                open_file.set(None);
                                                menu_open.set(false);

                                                // Clear config when switching vaults
                                                spawn(async move {
                                                    if let Err(e) = crate::ui::config::AppConfig::delete() {
                                                        tracing::warn!("Failed to delete config: {}", e);
                                                    }
                                                });
                                            },
                                            class: "flex w-full items-center gap-x-3 rounded-md p-3 text-sm font-semibold text-zinc-200 hover:bg-zinc-800",
                                            icons::SwitchIcon {}
                                            "Switch Vault"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Navigation bar
                nav { class: "bg-zinc-900 border-b border-zinc-700 px-4 py-3",
                    div { class: "flex items-center gap-4",
                        // Hamburger menu button
                        button {
                            onclick: move |_| menu_open.set(true),
                            class: "p-2 rounded-md text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-indigo-600",
                            span { class: "sr-only", "Open menu" }
                            icons::HamburgerIcon {}
                        }

                        // Command palette
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
                                                                    let mut open_file = open_file.clone();
                                                                    let mut current_screen = current_screen.clone();
                                                                    let mut palette_input = palette_input.clone();
                                                                    let mut palette_open = palette_open.clone();

                                                                    spawn(async move {
                                                                        match crate::core::note::parse_note(&file_path) {
                                                                            Ok(note) => {
                                                                                tracing::debug!("clicked on {} to open", file_name);
                                                                                open_file.set(Some(OpenFile {
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
                                div { class: "text-lg font-medium text-red-400", "Error" }
                                div { class: "text-sm text-zinc-200 mt-2", "{error}" }
                            }
                        }
                    },
                }
            }
        }
    }
}

fn use_load_last_session(vault_ctx: VaultContext, open_file: Signal<Option<OpenFile>>) {
    let mut config_loaded = use_signal(|| false);
    use_effect(move || {
        if !config_loaded() {
            config_loaded.set(true);
            load_last_session(vault_ctx.clone(), open_file.clone());
        }
    });
}

fn load_last_session(mut vault_ctx: VaultContext, mut open_file: Signal<Option<OpenFile>>) {
    spawn(async move {
        if let Some(config) = crate::ui::config::AppConfig::load() {
            if !config.validate_vault() {
                tracing::info!("Config vault path invalid, ignoring config");
                return;
            }
            vault_ctx.set_vault(config.last_vault_path.clone());

            if let Some(filename) = config.last_file {
                let file_path = config.last_vault_path.join(&filename);
                if file_path.exists() {
                    if let Ok(note) = crate::core::note::parse_note(&file_path) {
                        open_file.set(Some(OpenFile {
                            path: file_path,
                            filename: filename.clone(),
                            content: note.content,
                            share_with: note.frontmatter.share_with,
                            footnotes: note.frontmatter.footnotes,
                        }));
                    }
                }
            }
        }
    });
}
