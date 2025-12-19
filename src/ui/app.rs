use crate::ui::components::{
    command_palette::CommandPalette, icons, slide_over_menu::SlideOverMenu,
};
use crate::ui::context::VaultContext;
use crate::ui::screens::*;
use crate::ui::Screen;
use dioxus::prelude::*;
use std::path::PathBuf;

#[component]
pub fn App() -> Element {
    let vault_status = use_signal(|| VaultStatus::VaultNeeded);
    let current_screen = use_signal(|| Screen::Editor);
    let current_file = use_signal(|| None::<OpenFile>);
    let mut menu_open = use_signal(|| false);
    let palette_input = use_signal(|| String::new());
    let palette_open = use_signal(|| false);

    use_context_provider(|| VaultContext::new());
    let vault_ctx = use_context::<VaultContext>();

    use_load_last_session_on_start(vault_ctx, current_file);
    use_load_device_home_file_on_vault_change(vault_ctx, current_file);

    rsx! {
        document::Stylesheet { href: asset!("/assets/tailwind.css") }

        div { class: "h-screen flex flex-col bg-zinc-950",
            if vault_ctx.get_vault().is_some() {
                // Left slide-over menu
                if menu_open() {
                    SlideOverMenu {
                        menu_open,
                        current_screen,
                        current_file,
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
                        CommandPalette {
                            palette_input,
                            palette_open,
                            current_file,
                            current_screen,
                        }
                    }
                }

                // Main content area
                div { class: "flex-1 overflow-auto",
                    match current_screen() {
                        Screen::Editor => rsx! {
                            EditorScreen { current_file }
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

fn use_load_last_session_on_start(vault_ctx: VaultContext, current_file: Signal<Option<OpenFile>>) {
    let mut last_session_loaded = use_signal(|| false);
    use_effect(move || {
        if !last_session_loaded() {
            last_session_loaded.set(true);
            load_last_session(vault_ctx.clone(), current_file.clone());
        }
    });
}

fn load_last_session(mut vault_ctx: VaultContext, mut current_file: Signal<Option<OpenFile>>) {
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
                        current_file.set(Some(OpenFile {
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

fn use_load_device_home_file_on_vault_change(
    vault_ctx: VaultContext,
    current_file: Signal<Option<OpenFile>>,
) {
    use_effect(move || {
        if current_file.read().is_some() {
            tracing::debug!("file changed but file is valid, not a vault change");
            return;
        }
        if let Some(vault_path) = vault_ctx.get_vault() {
            load_device_home_file(vault_path, current_file.clone());
        };
    });
}

fn load_device_home_file(vault_path: PathBuf, mut current_file: Signal<Option<OpenFile>>) {
    let vault_path_for_spawn = vault_path.clone();
    spawn(async move {
        let device_name = match crate::core::device::get_local_device_name(&vault_path_for_spawn) {
            Ok(name) => name,
            Err(_) => return,
        };

        let home_filename = format!("home-{}.md", device_name);
        let home_path = vault_path_for_spawn.join(&home_filename);

        if let Ok(note) = crate::core::note::parse_note(&home_path) {
            current_file.set(Some(OpenFile {
                path: home_path,
                filename: home_filename,
                content: note.content,
                share_with: note.frontmatter.share_with,
                footnotes: note.frontmatter.footnotes,
            }));
        }
    });
}
