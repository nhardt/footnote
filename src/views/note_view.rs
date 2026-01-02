use crate::context::VaultContext;
use crate::model::note::Note;
use dioxus::prelude::*;
use std::path::PathBuf;

#[component]
pub fn NoteView(file_path: String) -> Element {
    let vault = use_context::<VaultContext>().get();

    let decoded = urlencoding::decode(&file_path).unwrap();
    let original_path = PathBuf::from(decoded.to_string());
    let mut note = match Note::from_path(original_path.clone()) {
        Ok(n) => n,
        Err(e) => return rsx! { "Could not load note {e}" },
    };

    let display_path = original_path
        .strip_prefix(vault.base_path())
        .unwrap_or(&original_path)
        .to_string_lossy()
        .to_string();

    let mut file_path_input = use_signal(|| display_path);
    let mut body = use_signal(|| note.content.clone());
    let mut share_with = use_signal(|| note.frontmatter.share_with.join(" "));
    let mut err_label = use_signal(|| String::new());

    let save_note = move |_| {
        let new_relative_path = file_path_input.read();
        let new_full_path = vault.base_path().join(&*new_relative_path);
        let share_with_str = share_with.read().clone();
        let share_with = share_with_str
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        note.frontmatter.share_with = share_with;
        if let Err(e) = note.update(&new_full_path, &body.read().clone()) {
            err_label.set(format!("Failed to save note: {e}"));
        }
    };

    rsx! {

        div { class: "h-full flex flex-col flex-1",
            div { class: "border-b border-zinc-800 bg-zinc-900/30 px-6 py-4",
                div { class: "max-w-5xl mx-auto",
                    div { class: "grid grid-cols-[auto_1fr] gap-x-4 gap-y-3 items-center",
                        label { class: "text-sm font-medium text-zinc-400", "File" }
                        input {
                            class: "px-3 py-1.5 bg-zinc-900 border border-zinc-700 rounded-md text-sm font-mono focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500",
                            r#type: "text",
                            value: "{file_path_input}",
                            oninput: move |e| file_path_input.set(e.value()),
                        }
                        label { class: "text-sm font-medium text-zinc-400", "Shared with" }
                        div { class: "flex items-center gap-2",
                            input {
                                class: "flex-1 px-3 py-1.5 bg-zinc-900 border border-zinc-700 rounded-md text-sm font-mono focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500",
                                r#type: "text",
                                value: "{share_with}",
                                oninput: move |e| share_with.set(e.value())
                            }
                            button { class: "px-4 py-1.5 bg-zinc-100 hover:bg-white hover:shadow-lg text-zinc-900 rounded-md text-sm font-medium transition-all",
                                onclick: save_note,
                                "Save"
                            }
                        }
                    }
                }
            }
            div { class: "h-full flex-1 overflow-hidden",
                div { class: "h-full max-w-5xl mx-auto px-6 py-6",
                    textarea {
                        class: "w-full h-full px-4 py-3 bg-zinc-900/30 border border-zinc-800 rounded-lg text-sm font-mono text-zinc-100 resize-none focus:border-zinc-700 focus:ring-1 focus:ring-zinc-700",
                        placeholder: "Start writing...",
                        value: "{body}",
                        oninput: move |e| body.set(e.value())
                    }
                }
            }
        }
    }
}
