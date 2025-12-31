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
        div { class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50",
            div {
                class: "bg-zinc-900 border border-zinc-800 rounded-lg shadow-2xl max-w-2xl w-full max-h-[90vh] flex flex-col",
                div { class: "p-6 border-b border-zinc-800",
                    h2 { class: "text-lg font-semibold font-mono", "Select Note" }
                    p { class: "text-sm text-zinc-500 mt-1",
                        "Select a file"
                    }
                }

                div { class: "p-6 border-b border-zinc-800 bg-zinc-900/50",
                    div { class: "flex items-center gap-2 mb-4",
                        div { class: "flex-1 px-3 py-2 bg-zinc-950 border border-zinc-800 rounded-md text-sm font-mono text-zinc-300 truncate",
                            "{current_path().display()}"
                        }
                        button { class: "px-3 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all flex items-center gap-2",
                            onclick: handle_go_up,
                            svg {
                                class: "w-4 h-4",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    d: "M5 15l7-7 7 7",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                }
                            }
                            "Up"
                        }
                    }
                    div { class: "flex gap-2",
                        button { class: "px-3 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all flex items-center gap-2",
                            onclick: handle_toggle_new_folder,
                            svg {
                                class: "w-4 h-4",
                                fill: "currentColor",
                                view_box: "0 0 20 20",
                                path { d: "M10.75 4.75a.75.75 0 0 0-1.5 0v4.5h-4.5a.75.75 0 0 0 0 1.5h4.5v4.5a.75.75 0 0 0 1.5 0v-4.5h4.5a.75.75 0 0 0 0-1.5h-4.5v-4.5Z" }
                            }
                            "Folder"
                        }
                        button { class: "px-3 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all flex items-center gap-2",
                            onclick: handle_toggle_new_file,
                            svg {
                                class: "w-4 h-4",
                                fill: "currentColor",
                                view_box: "0 0 20 20",
                                path { d: "M10.75 4.75a.75.75 0 0 0-1.5 0v4.5h-4.5a.75.75 0 0 0 0 1.5h4.5v4.5a.75.75 0 0 0 1.5 0v-4.5h4.5a.75.75 0 0 0 0-1.5h-4.5v-4.5Z" }
                            },
                            "File"
                        }
                    }
                }

                if show_new_folder_input() {
                    div { id: "new-folder-state",
                        div { class: "p-6 border-b border-zinc-800 bg-zinc-900/50",
                            div { class: "bg-zinc-800/30 border border-zinc-700 rounded-lg p-4",
                                label { class: "block text-sm font-medium text-zinc-300 mb-2",
                                    "New Folder Name"
                                }
                                div { class: "flex gap-2",
                                    input {
                                        class: "flex-1 px-3 py-2 bg-zinc-950 border border-zinc-700 rounded-md text-sm font-mono focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500",
                                        placeholder: "folder-name",
                                        r#type: "text",
                                        value: "{new_folder_name}",
                                        oninput: move |evt| new_folder_name.set(evt.value()),
                                    }
                                    button { class: "px-4 py-2 bg-zinc-100 hover:bg-white hover:shadow-lg text-zinc-900 rounded-md text-sm font-medium transition-all",
                                        disabled: new_folder_name().trim().is_empty(),
                                        onclick: handle_create_folder,
                                        "Create"
                                    }
                                    button { class: "px-4 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all",
                                        onclick: handle_toggle_new_folder,
                                        "Cancel"
                                    }
                                }
                            }
                        }
                    }
                }

                if show_new_file_input() {
                    div { id: "new-file-state",
                        div { class: "p-6 border-b border-zinc-800 bg-zinc-900/50",
                            div { class: "bg-zinc-800/30 border border-zinc-700 rounded-lg p-4",
                                label { class: "block text-sm font-medium text-zinc-300 mb-2",
                                    "New File Name"
                                }
                                div { class: "flex gap-2",
                                    input {
                                        class: "flex-1 px-3 py-2 bg-zinc-950 border border-zinc-700 rounded-md text-sm font-mono focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500",
                                        placeholder: "file-name.md",
                                        r#type: "text",
                                        value: "{new_file_name}",
                                        oninput: move |evt| new_file_name.set(evt.value()),
                                    }
                                    button { class: "px-4 py-2 bg-zinc-100 hover:bg-white hover:shadow-lg text-zinc-900 rounded-md text-sm font-medium transition-all",
                                        onclick: handle_create_file,
                                        disabled: new_file_name().trim().is_empty(),
                                        "Create"
                                    }
                                    button { class: "px-4 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all",
                                        onclick: handle_toggle_new_file,
                                        "Cancel"
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "flex-1 overflow-y-auto min-h-0 p-6",
                    div { class: "mb-6",
                        h3 { class: "text-xs font-semibold text-zinc-500 uppercase tracking-wider mb-3",
                            "Folders"
                        }
                        div { class: "space-y-1",
                            for folder in folders() {
                                {
                                    let folder_name = folder
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("?");
                                    let folder_path = folder.clone();
                                    rsx! {
                                        button { class: "w-full flex items-center gap-3 px-3 py-2 hover:bg-zinc-800/50 rounded-md text-left transition-colors group",
                                            svg {
                                                class: "w-5 h-5 text-zinc-500 group-hover:text-zinc-400",
                                                fill: "currentColor",
                                                view_box: "0 0 20 20",
                                                path { d: "M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" }
                                            }
                                            span {
                                                key: "{folder.display()}",
                                                class: "text-sm font-medium",
                                                onclick: move |_| current_path.set(folder_path.clone()),
                                                "{folder_name}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div {
                        h3 { class: "text-xs font-semibold text-zinc-500 uppercase tracking-wider mb-3",
                            "Files"
                        }
                        div { class: "space-y-1",
                            for file in files() {
                                {
                                    let file_name = file
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("?");
                                    rsx! {
                                        button { class: "w-full flex items-center gap-3 px-3 py-2 hover:bg-zinc-800/50 rounded-md text-left transition-colors group",
                                            svg {
                                                class: "w-5 h-5 text-zinc-500 group-hover:text-zinc-400",
                                                fill: "currentColor",
                                                view_box: "0 0 20 20",
                                                path {
                                                    clip_rule: "evenodd",
                                                    d: "M4 4a2 2 0 012-2h4.586A2 2 0 0112 2.586L15.414 6A2 2 0 0116 7.414V16a2 2 0 01-2 2H6a2 2 0 01-2-2V4z",
                                                    fill_rule: "evenodd",
                                                }
                                            }
                                            span { class: "text-sm",
                                                key: "{file_name}",
                                                onclick: move |_| on_select.call(file.clone()),
                                                "{file_name}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "p-6 border-t border-zinc-800 bg-zinc-900/50",
                        div { class: "flex gap-3",
                            button { class: "flex-1 px-4 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all",
                                onclick: handle_cancel,
                                "Cancel"
                            }
                            button { class: "flex-1 px-4 py-2 bg-zinc-100 hover:bg-white hover:shadow-lg text-zinc-900 rounded-md text-sm font-medium transition-all",
                                disabled: !path_is_valid,
                                onclick: handle_select_here,
                                "{action_label}"
                            }
                        }
                    }
                }
            }
        }
    }
}
