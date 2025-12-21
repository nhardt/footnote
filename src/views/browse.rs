use dioxus::prelude::*;
use std::path::PathBuf;
use urlencoding;

use crate::components::PlainTextViewer;
use footnote::core::note;

#[component]
pub fn Browse(file_path: String) -> Element {
    let path = PathBuf::from(&file_path);
    let path_clone = path.clone();

    // Load and parse the note
    let note_result = use_resource(move || {
        let path = path_clone.clone();
        async move {
            note::parse_note(&path)
        }
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
                            footnotes: note.footnotes.clone(),
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
