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
                                    footnotes: note.frontmatter.footnotes,
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
                                        svg {
                                            view_box: "0 0 24 24",
                                            fill: "none",
                                            stroke: "currentColor",
                                            stroke_width: "1.5",
                                            "aria-hidden": "true",
                                            class: "size-6",
                                            path {
                                                d: "M6 18 18 6M6 6l12 12",
                                                stroke_linecap: "round",
                                                stroke_linejoin: "round"
                                            }
                                        }
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
                                            svg {
                                                view_box: "0 0 24 24",
                                                fill: "none",
                                                stroke: "currentColor",
                                                stroke_width: "1.5",
                                                class: "size-6 shrink-0 text-zinc-400",
                                                path {
                                                    d: "M16.862 4.487l1.687-1.688a1.875 1.875 0 112.652 2.652L10.582 16.07a4.5 4.5 0 01-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 011.13-1.897l8.932-8.931zm0 0L19.5 7.125M18 14v4.75A2.25 2.25 0 0115.75 21H5.25A2.25 2.25 0 013 18.75V8.25A2.25 2.25 0 015.25 6H10",
                                                    stroke_linecap: "round",
                                                    stroke_linejoin: "round"
                                                }
                                            }
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
                                            svg {
                                                view_box: "0 0 24 24",
                                                fill: "none",
                                                stroke: "currentColor",
                                                stroke_width: "1.5",
                                                class: "size-6 shrink-0 text-zinc-400",
                                                path {
                                                    d: "M15 19.128a9.38 9.38 0 002.625.372 9.337 9.337 0 004.121-.952 4.125 4.125 0 00-7.533-2.493M15 19.128v-.003c0-1.113-.285-2.16-.786-3.07M15 19.128v.106A12.318 12.318 0 018.624 21c-2.331 0-4.512-.645-6.374-1.766l-.001-.109a6.375 6.375 0 0111.964-3.07M12 6.375a3.375 3.375 0 11-6.75 0 3.375 3.375 0 016.75 0zm8.25 2.25a2.625 2.625 0 11-5.25 0 2.625 2.625 0 015.25 0z",
                                                    stroke_linecap: "round",
                                                    stroke_linejoin: "round"
                                                }
                                            }
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
                                            svg {
                                                view_box: "0 0 24 24",
                                                fill: "none",
                                                stroke: "currentColor",
                                                stroke_width: "1.5",
                                                class: "size-6 shrink-0 text-zinc-400",
                                                path {
                                                    d: "M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0l3.181 3.183a8.25 8.25 0 0013.803-3.7M4.031 9.865a8.25 8.25 0 0113.803-3.7l3.181 3.182m0-4.991v4.99",
                                                    stroke_linecap: "round",
                                                    stroke_linejoin: "round"
                                                }
                                            }
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
                                            svg {
                                                view_box: "0 0 24 24",
                                                fill: "none",
                                                stroke: "currentColor",
                                                stroke_width: "1.5",
                                                class: "size-6 shrink-0 text-zinc-400",
                                                path {
                                                    d: "M7.5 21L3 16.5m0 0L7.5 12M3 16.5h13.5m0-13.5L21 7.5m0 0L16.5 12M21 7.5H7.5",
                                                    stroke_linecap: "round",
                                                    stroke_linejoin: "round"
                                                }
                                            }
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
                            svg {
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "1.5",
                                "aria-hidden": "true",
                                class: "size-6",
                                path {
                                    d: "M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round"
                                }
                            }
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
