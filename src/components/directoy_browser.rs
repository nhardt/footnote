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
        div { class: "directory-browser",
            div { class: "card",
                div { class: "header",
                    h1 { "Select Directory" }
                }

                div { class: "content",
                    div { class: "field",
                        label { "Current Path" }
                        div { class: "button-row",
                            div { class: "path-display",
                                "{current_path().display()}"
                            }
                            button {
                                class: "secondary",
                                onclick: handle_go_up,
                                "↑ Up"
                            }
                            button {
                                class: "secondary",
                                onclick: handle_toggle_new_folder,
                                "+ Folder"
                            }
                        }
                    }

                    if show_new_folder_input() {
                        div { class: "field",
                            label { "New Folder Name" }
                            div { class: "button-row",
                                input {
                                    r#type: "text",
                                    placeholder: "folder-name",
                                    value: "{new_folder_name}",
                                    oninput: move |evt| new_folder_name.set(evt.value()),
                                    autofocus: true,
                                }
                                button {
                                    disabled: new_folder_name().trim().is_empty(),
                                    onclick: handle_create_folder,
                                    "Create"
                                }
                            }
                        }
                    }

                    div { class: "folder-list",
                        if folders().is_empty() {
                            div { class: "empty", "No subdirectories" }
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
                                            class: "list-item",
                                            onclick: move |_| current_path.set(folder_path.clone()),
                                            "📁 {folder_name}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "footer",
                    button {
                        class: "secondary",
                        onclick: handle_cancel,
                        "Cancel"
                    }
                    button {
                        disabled: !path_is_valid,
                        onclick: handle_select_here,
                        "{action_label}"
                    }
                }
            }
        }
    }
}
