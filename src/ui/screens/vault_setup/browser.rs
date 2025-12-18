use dioxus::prelude::*;
use std::path::PathBuf;
use super::needed::VaultStatus;
use tracing;

#[component]
pub fn DirectoryBrowserScreen(mut vault_status: Signal<VaultStatus>, action: &'static str) -> Element {
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
            div { class: "max-w-2xl w-full bg-app-panel rounded-lg shadow-lg",
                div { class: "p-6 border-b border-app-border",
                    h1 { class: "text-2xl font-bold text-app-text text-center", "Select Directory" }
                }

                div { class: "p-6",
                    div { class: "mb-4",
                        label { class: "block text-sm font-medium text-app-text-secondary mb-2", "Current Path" }
                        div { class: "flex gap-2",
                            div { class: "flex-1 px-3 py-2 border border-app-border-subtle rounded-md bg-app-hover text-app-text font-mono text-sm break-all",
                                "{current_path().display()}"
                            }
                            button {
                                class: "px-3 py-2 bg-app-panel border border-app-border-subtle rounded-md hover:bg-app-hover",
                                onclick: handle_go_up,
                                "↑ Up"
                            }
                            button {
                                class: "px-3 py-2 bg-app-panel border border-app-border-subtle rounded-md hover:bg-app-hover",
                                onclick: handle_toggle_new_folder,
                                "+ Folder"
                            }
                        }
                    }

                    if show_new_folder_input() {
                        div { class: "mb-4",
                            label { class: "block text-sm font-medium text-app-text-secondary mb-2", "New Folder Name" }
                            div { class: "flex gap-2",
                                input {
                                    r#type: "text",
                                    class: "flex-1 px-3 py-2 border border-app-border-subtle rounded-md focus:outline-none focus:ring-2 focus:ring-app-primary",
                                    placeholder: "folder-name",
                                    value: "{new_folder_name}",
                                    oninput: move |evt| new_folder_name.set(evt.value()),
                                    autofocus: true,
                                }
                                button {
                                    class: "px-3 py-2 bg-app-primary text-white rounded-md hover:bg-app-primary-hover disabled:bg-app-hover disabled:cursor-not-allowed",
                                    disabled: new_folder_name().trim().is_empty(),
                                    onclick: handle_create_folder,
                                    "Create"
                                }
                            }
                        }
                    }

                    div { class: "mb-4 max-h-96 overflow-y-auto border border-app-border rounded-md",
                        if folders().is_empty() {
                            div { class: "p-4 text-center text-app-text-muted", "No subdirectories" }
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
                                            class: "px-4 py-2 hover:bg-app-hover cursor-pointer border-b border-app-border last:border-b-0",
                                            onclick: move |_| current_path.set(folder_path.clone()),
                                            "📁 {folder_name}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "p-6 border-t border-app-border flex gap-2",
                    button {
                        class: "flex-1 px-4 py-2 bg-app-panel text-app-text-secondary border border-app-border-subtle rounded-md hover:bg-app-hover",
                        onclick: handle_cancel,
                        "Cancel"
                    }
                    button {
                        class: if (action == "Open" && !has_footnotes_dir()) || (action == "Join" && has_footnotes_dir()) || (action == "Create" && has_footnotes_dir()) {
                            "flex-1 px-4 py-2 bg-app-hover text-app-text-muted rounded-md cursor-not-allowed"
                        } else {
                            "flex-1 px-4 py-2 bg-app-primary text-white rounded-md hover:bg-app-primary-hover"
                        },
                        disabled: (action == "Open" && !has_footnotes_dir()) || (action == "Join" && has_footnotes_dir()) || (action == "Create" && has_footnotes_dir()),
                        onclick: handle_select_here,
                        "{action} Here"
                    }
                }
            }
        }
    }
}
