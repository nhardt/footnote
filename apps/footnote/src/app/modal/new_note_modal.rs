use dioxus::prelude::*;

use chrono::Local;

use crate::context::AppContext;
use crate::context::MenuContext;

#[component]
pub fn NewNoteModal() -> Element {
    let app_context = use_context::<AppContext>();
    let mut note_path = use_signal(|| String::new());
    let mut error_message = use_signal(|| String::new());

    let create_by_path = move |_| async move {
        let path_str = note_path.read().trim().to_string();
        if path_str.is_empty() {
            error_message.set("Path cannot be empty".to_string());
            return;
        }

        let full_path = app_context.vault.read().base_path().join(&path_str);

        if let Some(parent) = full_path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                error_message.set(format!("Failed to create directory: {}", e));
                return;
            }
        }

        if let Err(e) = app_context.vault.read().note_create(&full_path, "") {
            error_message.set(format!("Failed to create note: {}", e));
            return;
        }

        consume_context::<MenuContext>().go_note(&path_str);
    };

    let create_now_note = move |_| async move {
        let now = Local::now();
        let path_str = now.format("%Y/%m/%d/%H_%M_%S.md").to_string();
        let full_path = app_context.vault.read().base_path().join(&path_str);

        if let Some(parent) = full_path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                error_message.set(format!("Failed to create directory: {}", e));
                return;
            }
        }

        if let Err(e) = app_context.vault.read().note_create(&full_path, "") {
            error_message.set(format!("Failed to create note: {}", e));
            return;
        }

        consume_context::<MenuContext>().go_note(&path_str);
    };

    rsx! {
        div {
            class: "fixed text-zinc-100 inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50",
            onclick: move |_| consume_context::<MenuContext>().close_all(),

            div {
                class: "bg-zinc-900 border border-zinc-800 rounded-lg shadow-2xl max-w-md w-full",
                onclick: move |evt| evt.stop_propagation(),

                div { class: "p-6 border-b border-zinc-800",
                    h2 { class: "text-lg font-semibold", "Create New Note" }
                }

                div { class: "p-6 space-y-6",
                    div { class: "space-y-3",
                        label { class: "block text-sm font-medium text-zinc-300",
                            "Create by path"
                        }
                        div { class: "flex gap-2",
                            input {
                                class: "flex-1 px-3 py-2 bg-zinc-950 border border-zinc-700 rounded-md text-sm font-mono focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500",
                                placeholder: "path/to/note.md",
                                r#type: "text",
                                value: "{note_path}",
                                oninput: move |evt| {
                                    note_path.set(evt.value());
                                    error_message.set(String::new());
                                },
                            }
                            button {
                                class: "px-4 py-2 bg-zinc-100 hover:bg-white hover:shadow-lg text-zinc-900 rounded-md text-sm font-medium transition-all",
                                disabled: note_path.read().trim().is_empty(),
                                onclick: create_by_path,
                                "Create"
                            }
                        }
                        if !error_message.read().is_empty() {
                            p { class: "text-sm text-red-400",
                                "{error_message}"
                            }
                        }
                    }

                    div { class: "relative",
                        div { class: "absolute inset-0 flex items-center",
                            div { class: "w-full border-t border-zinc-700" }
                        }
                        div { class: "relative flex justify-center text-xs uppercase",
                            span { class: "bg-zinc-900 px-2 text-zinc-500", "or" }
                        }
                    }

                    div { class: "space-y-3",
                        label { class: "block text-sm font-medium text-zinc-300",
                            "Quick capture"
                        }
                        button {
                            class: "w-full px-4 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all text-left flex items-center gap-2",
                            onclick: create_now_note,
                            svg {
                                class: "w-4 h-4 text-zinc-400",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    d: "M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                }
                            }
                            span { "Create timestamped note" }
                        }
                        p { class: "text-xs text-zinc-500",
                            "Creates: {Local::now().format(\"%Y/%m/%d/%H_%M_%S.md\")}"
                        }
                    }
                }

                div { class: "p-6 border-t border-zinc-800",
                    button {
                        class: "w-full px-4 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all",
                        onclick: move |_| consume_context::<MenuContext>().close_all(),
                        "Cancel"
                    }
                }
            }
        }
    }
}
