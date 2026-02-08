use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    rsx! {
        "Welcome to Footnote, a place to explore and learn"
    }

    // Maybe something like:
    //
    // New Note
    // Recent notes
    // - note list
    // Contacts
    // - contact list, maybe sorted by ones with new things to read
}
