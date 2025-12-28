use dioxus::prelude::*;
use footnote::model::note::Note;
use std::path::PathBuf;

#[component]
pub fn NoteView(file_path: String) -> Element {
    let decoded = urlencoding::decode(&file_path).unwrap();
    let path = PathBuf::from(decoded.to_string());
    let note = match Note::from_path(path.clone()) {
        Ok(n) => n,
        Err(_) => return rsx! { "Could not load note" },
    };

    let mut body = use_signal(|| note.content.clone());
    let mut share_with = use_signal(|| String::new());

    let save_note = move |_| {
        let mut updated_note = note.clone();
        updated_note.content = body.read().clone();
        updated_note.save(&path).unwrap();
    };

    rsx! {
        div { class: "flex flex-col h-full w-2xl gap-2",
            div { class: "grid grid-cols-[auto_1fr] gap-4",
                label { "Title" }
                input {
                    class: "border-1",
                    r#type: "text",
                    value: "{decoded}",
                }
                label { "Shared with:" }
                input {
                    class: "border-1",
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
        }
    }
}
