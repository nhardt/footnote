use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
enum TextSegment {
    Text(String),
    Footnote(usize),
}

#[component]
pub fn PlainTextViewer(content: String, on_footnote_click: EventHandler<uuid::Uuid>) -> Element {
    rsx! {
        {content}
    }
}
