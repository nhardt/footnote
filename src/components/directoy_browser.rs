use dioxus::prelude::*;
use std::path::{Path, PathBuf};
use tracing;

#[component]
pub fn DirectoryBrowser(
    base_path: PathBuf,
    only_directories: bool,
    on_select: EventHandler<PathBuf>,
    on_file_create: EventHandler<PathBuf>,
    on_cancel: EventHandler<()>,
    action_label: String,
    #[props(default)] is_valid: Option<Callback<PathBuf, bool>>,
) -> Element {
    let mut current_path = use_signal(|| base_path.to_path_buf());

    let mut folders = use_signal(|| Vec::<PathBuf>::new());
    let mut new_folder_name = use_signal(|| String::new());
    let mut show_new_folder_input = use_signal(|| false);

    let mut files = use_signal(|| Vec::<PathBuf>::new());
    let mut new_file_name = use_signal(|| String::new());
    let mut show_new_file_input = use_signal(|| false);

    let handle_select_here = move |_| {
        on_select.call(current_path());
    };

    let handle_cancel = move |_| {
        on_cancel.call(());
    };

    let path_is_valid = is_valid
        .as_ref()
        .map_or(true, |validator| validator.call(current_path()));

    let handle_go_up = move |_| {
        if let Some(parent) = current_path().parent() {
            current_path.set(parent.to_path_buf());
        }
    };

    let mut refresh_trigger = use_signal(|| 0);
    use_effect(move || {
        let path = current_path();
        let _ = refresh_trigger();

        spawn(async move {
            let mut folder_list = Vec::new();
            let mut file_list = Vec::new();
            if let Ok(entries) = std::fs::read_dir(&path) {
                for entry in entries.flatten() {
                    if entry.file_name().to_string_lossy().starts_with('.') {
                        continue;
                    }

                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_dir() {
                            folder_list.push(entry.path());
                            if only_directories {
                                continue;
                            }
                        }

                        if metadata.is_file() {
                            file_list.push(entry.path());
                        }
                    }
                }
            }
            folder_list.sort();
            file_list.sort();
            folders.set(folder_list);
            files.set(file_list);
        });
    });

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

    let handle_create_file = move |_| {
        if new_file_name().trim().is_empty() {
            return;
        }

        let file_name = new_file_name().trim().to_string();
        let new_path = current_path().join(&file_name);
        on_file_create(new_path);

        refresh_trigger += 1;
        new_file_name.set(String::new());
        show_new_file_input.set(false);
    };

    let handle_toggle_new_file = move |_| {
        show_new_file_input.set(!show_new_file_input());
        if show_new_file_input() {
            new_file_name.set(String::new());
        }
    };

    rsx! {
        div { class: "flex flex-col max-w-2xl w-full border-1 gap-4 p-4",
            h1 { class: "text-center bg-gray-700", "Select Directory" }

            div { class: "flex w-full border-1 gap-4 p-2 m-2",
                div { class: "flex-1", "{current_path().display()}" }
                button { class: "border-1 rounded w-24", onclick: handle_go_up, "↑ Up" }
                button { class: "border-1 rounded w-24", onclick: handle_toggle_new_folder, "+ Folder" }
                button { class: "border-1 rounded w-24", onclick: handle_toggle_new_file, "+ File" }
            }

            if show_new_folder_input() {
                div { class: "grid grid-cols-[1fr_auto] p-4",
                    h2 { class: "col-span-2 text-center bg-gray-600", "New Folder Name" }

                    input {
                        class: "border-1",
                        r#type: "text",
                        placeholder: "folder-name",
                        value: "{new_folder_name}",
                        oninput: move |evt| new_folder_name.set(evt.value()),
                        autofocus: true,
                    }
                    button {
                        class: "border-1 w-24",
                        disabled: new_folder_name().trim().is_empty(),
                        onclick: handle_create_folder,
                        "Create"
                    }
                }
            }

            if show_new_file_input() {
                div { class: "grid grid-cols-[1fr_auto] p-4",
                    h2 { class: "col-span-2 text-center bg-gray-600", "New File Name" }

                    input {
                        class: "border-1",
                        r#type: "text",
                        placeholder: "file-name",
                        value: "{new_file_name}",
                        oninput: move |evt| new_file_name.set(evt.value()),
                        autofocus: true,
                    }
                    button {
                        class: "border-1 w-24",
                        disabled: new_file_name().trim().is_empty(),
                        onclick: handle_create_file,
                        "Create"
                    }
                }
            }

            // Folder list
            div {
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
                                class: "font-bold",
                                onclick: move |_| current_path.set(folder_path.clone()),
                                "{folder_name}"
                            }
                        }
                    }
                }
            }

            // File list
            div { class: "border-1 p-4",
                for file in files() {
                    {
                        let file_name = file
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("?");
                        rsx! {
                            div {
                                key: "{file_name}",
                                onclick: move |_| on_select.call(file.clone()),
                                "{file_name}"
                            }
                        }
                    }
                }
            }

            button { class: "border-1 rounded", disabled: !path_is_valid, onclick: handle_select_here, "{action_label}" }
            button { class: "border-1 rounded", onclick: handle_cancel, "Cancel" }
        }
    }
}
