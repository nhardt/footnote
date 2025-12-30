use crate::context::VaultContext;
use dioxus::prelude::*;
use footnote::model::note::Note;
use std::path::PathBuf;

#[component]
pub fn NoteView(file_path: String) -> Element {
    let vault_ctx = use_context::<VaultContext>();
    let vault_path = vault_ctx.get_vault().expect("vault not set in context!");

    let decoded = urlencoding::decode(&file_path).unwrap();
    let original_path = PathBuf::from(decoded.to_string());
    let mut note = match Note::from_path(original_path.clone()) {
        Ok(n) => n,
        Err(_) => return rsx! { "Could not load note" },
    };

    let display_path = original_path
        .strip_prefix(&vault_path)
        .unwrap_or(&original_path)
        .to_string_lossy()
        .to_string();

    let mut file_path_input = use_signal(|| display_path);
    let mut body = use_signal(|| note.content.clone());
    let mut share_with = use_signal(|| note.frontmatter.share_with.join(" "));
    let mut err_label = use_signal(|| String::new());

    let save_note = move |_| {
        let new_relative_path = file_path_input.read();
        let new_full_path = vault_path.join(&*new_relative_path);
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
        div { class: "flex flex-col h-full w-2xl gap-2",
            div { class: "grid grid-cols-[auto_1fr] gap-4",
                label { "File" }
                input {
                    class: "border-1 px-2",
                    r#type: "text",
                    value: "{file_path_input}",
                    oninput: move |e| file_path_input.set(e.value()),
                }
                label { "Shared with:" }
                input {
                    class: "border-1 px-2",
                    r#type: "text",
                    value: "{share_with}",
                    oninput: move |e| share_with.set(e.value())
                }
            }
            textarea {
                class: "flex-1 w-full border-1 p-4",
                value: "{body}",
                oninput: move |e| body.set(e.value())
            }
            button {
                class: "border-1 p-4 my-4",
                onclick: save_note,
                "Save"
            }
            label { "{err_label}" }
        }
    }
}
