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

    // Signals for each field
    //let mut title = use_signal(|| note.title.clone());
    let mut body = use_signal(|| note.content.clone());
    let mut share_with = use_signal(|| String::new());

    let save_note = move |_| {
        let mut updated_note = note.clone();
        //updated_note.title = title.read().clone();
        updated_note.content = body.read().clone();
        updated_note.save(&path).unwrap();
    };

    rsx! {
        div { class: "flex flex-col h-full max-w-2xl",
            h1 { class:"text-3xl font-bold", "{decoded}" }
            div { class: "grid grid-cols-2",
                label { "Title" }
                input {
                    class: "border-2",
                    r#type: "text",
                    value: "{decoded}",
                }
                label { "Shared with:" }
                input {
                    class: "border-2",
                    r#type: "text",
                    value: "{share_with}",
                    oninput: move |e| share_with.set(e.value())
                }
            }
            textarea {
                class: "flex-1 w-full border-2",
                value: "{body}",
                oninput: move |e| body.set(e.value())
            }
            button {
                class: "border-2 p-4 my-4",
                onclick: save_note,
                "Save"
            }
        }
    }
}
