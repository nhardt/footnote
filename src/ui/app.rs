use dioxus::prelude::*;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq)]
enum Screen {
    Editor,
    Contacts,
}

#[derive(Clone, PartialEq)]
enum VaultStatus {
    Initializing,
    Ready(PathBuf),
    Error(String),
}

#[component]
pub fn App() -> Element {
    let mut current_screen = use_signal(|| Screen::Editor);
    let mut vault_status = use_signal(|| VaultStatus::Initializing);

    // Initialize vault on first render
    use_effect(move || {
        spawn(async move {
            let home_dir = match dirs::home_dir() {
                Some(dir) => dir,
                None => {
                    vault_status.set(VaultStatus::Error("Could not find home directory".to_string()));
                    return;
                }
            };

            let vault_path = home_dir.join("footnotes");

            // Check if vault already exists
            let footnotes_dir = vault_path.join(".footnotes");
            if footnotes_dir.exists() {
                vault_status.set(VaultStatus::Ready(vault_path));
                return;
            }

            // Initialize new vault
            match crate::core::init::init(
                Some(vault_path.clone()),
                Some("me"),
                Some("primary")
            ).await {
                Ok(_) => vault_status.set(VaultStatus::Ready(vault_path)),
                Err(e) => vault_status.set(VaultStatus::Error(format!("Failed to initialize vault: {}", e))),
            }
        });
    });

    rsx! {
        document::Stylesheet { href: asset!("/assets/tailwind.css") }

        div { class: "h-screen flex flex-col bg-gray-50",
            match vault_status() {
                VaultStatus::Initializing => rsx! {
                    div { class: "flex items-center justify-center h-full",
                        div { class: "text-center",
                            div { class: "text-lg font-medium text-gray-700", "Initializing vault..." }
                            div { class: "text-sm text-gray-500 mt-2", "Setting up ~/footnotes/" }
                        }
                    }
                },
                VaultStatus::Error(ref error) => rsx! {
                    div { class: "flex items-center justify-center h-full",
                        div { class: "text-center max-w-md",
                            div { class: "text-lg font-medium text-red-600", "Error" }
                            div { class: "text-sm text-gray-700 mt-2", "{error}" }
                        }
                    }
                },
                VaultStatus::Ready(ref _path) => rsx! {
                    // Navigation bar
                    nav { class: "bg-white border-b border-gray-200 px-4 py-3",
                        div { class: "flex gap-4",
                            button {
                                class: if current_screen() == Screen::Editor { "px-4 py-2 font-medium text-blue-600 border-b-2 border-blue-600" } else { "px-4 py-2 font-medium text-gray-600 hover:text-gray-900" },
                                onclick: move |_| current_screen.set(Screen::Editor),
                                "Editor"
                            }
                            button {
                                class: if current_screen() == Screen::Contacts { "px-4 py-2 font-medium text-blue-600 border-b-2 border-blue-600" } else { "px-4 py-2 font-medium text-gray-600 hover:text-gray-900" },
                                onclick: move |_| current_screen.set(Screen::Contacts),
                                "Contacts"
                            }
                        }
                    }

                    // Main content area
                    div { class: "flex-1 overflow-auto",
                        match current_screen() {
                            Screen::Editor => rsx! {
                                EditorScreen {}
                            },
                            Screen::Contacts => rsx! {
                                ContactsScreen {}
                            },
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn EditorScreen() -> Element {
    let mut doc_title = use_signal(|| String::new());
    let mut content = use_signal(|| String::new());

    rsx! {
        div { class: "max-w-4xl mx-auto p-6 space-y-4",
            // Document title
            div {
                label { class: "block text-sm font-medium text-gray-700 mb-2", "Document Title" }
                input {
                    r#type: "text",
                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                    placeholder: "Untitled",
                    value: "{doc_title}",
                    oninput: move |evt| doc_title.set(evt.value()),
                }
            }

            // Share with
            div {
                label { class: "block text-sm font-medium text-gray-700 mb-2", "Share With" }
                div { class: "px-3 py-2 border border-gray-300 rounded-md bg-gray-50 text-gray-500",
                    "[ list of contacts ]"
                }
            }

            // Text editor
            div {
                label { class: "block text-sm font-medium text-gray-700 mb-2", "Content" }
                textarea {
                    class: "w-full h-96 px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono",
                    placeholder: "Start writing...",
                    value: "{content}",
                    oninput: move |evt| content.set(evt.value()),
                }
            }
        }
    }
}

#[component]
fn ContactsScreen() -> Element {
    rsx! {
        div { class: "max-w-4xl mx-auto p-6",
            div { class: "text-center text-gray-500 text-lg", "[ Contacts Placeholder ]" }
        }
    }
}
