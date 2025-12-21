use dioxus::prelude::*;
use std::path::PathBuf;
use tracing;

#[component]
pub fn DirectoryBrowser(
    on_select: EventHandler<PathBuf>,
    on_cancel: EventHandler<()>,
    action_label: String,
    #[props(default)] is_valid: Option<Callback<PathBuf, bool>>,
) -> Element {
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
        });
    });

    // Check if current path is valid
    let path_is_valid = is_valid
        .as_ref()
        .map_or(true, |validator| validator.call(current_path()));

    let handle_go_up = move |_| {
        if let Some(parent) = current_path().parent() {
            current_path.set(parent.to_path_buf());
        }
    };

    let handle_select_here = move |_| {
        on_select.call(current_path());
    };

    let handle_cancel = move |_| {
        on_cancel.call(());
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
            div { class: "max-w-2xl w-full bg-zinc-800 rounded-lg shadow-lg",
                div { class: "p-6 border-b border-zinc-700",
                    h1 { class: "text-2xl font-bold text-zinc-100 text-center", "Select Directory" }
                }

                div { class: "p-6",
                    div { class: "mb-4",
                        label { class: "block text-sm font-medium text-zinc-200 mb-2", "Current Path" }
                        div { class: "flex gap-2",
                            div { class: "flex-1 px-3 py-2 border border-zinc-600 rounded-md bg-zinc-700 text-zinc-100 font-mono text-sm break-all",
                                "{current_path().display()}"
                            }
                            button {
                                class: "px-3 py-2 bg-zinc-800 border border-zinc-600 rounded-md hover:bg-zinc-700",
                                onclick: handle_go_up,
                                "↑ Up"
                            }
                            button {
                                class: "px-3 py-2 bg-zinc-800 border border-zinc-600 rounded-md hover:bg-zinc-700",
                                onclick: handle_toggle_new_folder,
                                "+ Folder"
                            }
                        }
                    }

                    if show_new_folder_input() {
                        div { class: "mb-4",
                            label { class: "block text-sm font-medium text-zinc-200 mb-2", "New Folder Name" }
                            div { class: "flex gap-2",
                                input {
                                    r#type: "text",
                                    class: "flex-1 px-3 py-2 border border-zinc-600 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-600",
                                    placeholder: "folder-name",
                                    value: "{new_folder_name}",
                                    oninput: move |evt| new_folder_name.set(evt.value()),
                                    autofocus: true,
                                }
                                button {
                                    class: "px-3 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-700 disabled:bg-zinc-700 disabled:cursor-not-allowed",
                                    disabled: new_folder_name().trim().is_empty(),
                                    onclick: handle_create_folder,
                                    "Create"
                                }
                            }
                        }
                    }

                    div { class: "mb-4 max-h-96 overflow-y-auto border border-zinc-700 rounded-md",
                        if folders().is_empty() {
                            div { class: "p-4 text-center text-zinc-400", "No subdirectories" }
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
                                            class: "px-4 py-2 hover:bg-zinc-700 cursor-pointer border-b border-zinc-700 text-zinc-200 last:border-b-0",
                                            onclick: move |_| current_path.set(folder_path.clone()),
                                            "📁 {folder_name}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "p-6 border-t border-zinc-700 flex gap-2",
                    button {
                        class: "flex-1 px-4 py-2 bg-zinc-800 text-zinc-200 border border-zinc-600 rounded-md hover:bg-zinc-700",
                        onclick: handle_cancel,
                        "Cancel"
                    }
                    button {
                        class: if !path_is_valid {
                            "flex-1 px-4 py-2 bg-zinc-700 text-zinc-400 rounded-md cursor-not-allowed"
                        } else {
                            "flex-1 px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-700"
                        },
                        disabled: !path_is_valid,
                        onclick: handle_select_here,
                        "{action_label}"
                    }
                }
            }
        }
    }
}
