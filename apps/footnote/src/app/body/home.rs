use dioxus::prelude::*;

use crate::context::MenuContext;

#[component]
pub fn Home() -> Element {
    rsx! {
        div {
            class: "flex-1 overflow-y-auto",
            div {
                class: "max-w-3xl mx-auto px-6 py-12",

                div {
                    class: "mb-12",
                    h1 {
                        class: "text-2xl font-bold font-mono text-zinc-100 mb-2",
                        "Footnote"
                    }
                    p {
                        class: "text-sm text-zinc-400",
                        "Local-first notes with trusted networks"
                    }
                }

                div {
                    class: "space-y-8",

                    // Quick actions
                    div {
                        class: "border border-zinc-800 rounded-lg bg-zinc-900/30 p-6",
                        h2 {
                            class: "text-sm font-semibold font-mono text-zinc-400 mb-4",
                            "Quick Actions"
                        }
                        div {
                            class: "space-y-2",
                            button {
                                class: "w-full px-4 py-3 text-left text-sm text-zinc-300 hover:bg-zinc-800 hover:text-zinc-100 rounded-lg transition-colors border border-zinc-800",
                                onclick: move |_| consume_context::<MenuContext>().set_new_note_visible(),
                                "New Note"
                            }
                            button {
                                class: "w-full px-4 py-3 text-left text-sm text-zinc-300 hover:bg-zinc-800 hover:text-zinc-100 rounded-lg transition-colors border border-zinc-800",
                                "Import Contact"
                            }
                        }
                    }

                    // Recent notes placeholder
                    div {
                        class: "border border-zinc-800 rounded-lg bg-zinc-900/30",
                        div {
                            class: "px-6 py-4 border-b border-zinc-800",
                            h2 {
                                class: "text-sm font-semibold font-mono text-zinc-400",
                                "Recent Notes"
                            }
                        }
                        div {
                            class: "px-6 py-8 text-sm text-zinc-500 text-center",
                            "Use the search bar to find notes, or create a new one from the menu"
                        }
                    }

                    // Contacts placeholder
                    div {
                        class: "border border-zinc-800 rounded-lg bg-zinc-900/30",
                        div {
                            class: "px-6 py-4 border-b border-zinc-800",
                            h2 {
                                class: "text-sm font-semibold font-mono text-zinc-400",
                                "Contacts"
                            }
                        }
                        div {
                            class: "px-6 py-8 text-sm text-zinc-500 text-center",
                            "No contacts yet"
                        }
                    }
                }
            }
        }
    }
}
