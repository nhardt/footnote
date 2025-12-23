use dioxus::prelude::*;
use urlencoding;

use crate::components::PlainTextViewer;
use crate::context::VaultContext;
use crate::model::note::Note;

#[component]
pub fn Browse(file_path: String) -> Element {
    let vault_ctx = use_context::<VaultContext>();
    let vault_path = vault_ctx.get_vault().unwrap_or_default();

    let path = vault_path.join(&file_path);
    let path_clone = path.clone();

    // Load and parse the note
    let note_result = use_resource(move || {
        let path = path_clone.clone();
        async move { Note::from_path(path) }
    });

    rsx! {
        div {
            div {
                div {
                    {path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Untitled")}
                }
                Link {
                    to: format!("/edit?file={}", urlencoding::encode(&file_path)),
                    "Edit"
                }
            }

            div {
                match &*note_result.read_unchecked() {
                    Some(Ok(note)) => rsx! {
                        PlainTextViewer {
                            content: note.content.clone(),
                            on_footnote_click: move |uuid| {
                                println!("Clicked footnote with UUID: {}", uuid);
                            }
                        }
                    },
                    Some(Err(e)) => rsx! {
                        div { "Error loading note: {e}" }
                    },
                    None => rsx! {
                        div { "Loading..." }
                    }
                }
            }
        }
    }
}
