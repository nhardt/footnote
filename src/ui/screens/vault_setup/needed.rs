use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub enum VaultStatus {
    VaultNeeded,
    BrowsingToCreate,
    BrowsingToOpen,
    BrowsingToJoin,
    Creating {
        vault_path: std::path::PathBuf,
    },
    Opening {
        vault_path: std::path::PathBuf,
    },
    Joining {
        vault_path: std::path::PathBuf,
        device_name: String,
        connect_url: String,
    },
    Error(String),
}

#[component]
pub fn VaultNeededScreen(mut vault_status: Signal<VaultStatus>) -> Element {
    let handle_create = move |_| {
        vault_status.set(VaultStatus::BrowsingToCreate);
    };

    let handle_join = move |_| {
        vault_status.set(VaultStatus::BrowsingToJoin);
    };

    let handle_open = move |_| {
        vault_status.set(VaultStatus::BrowsingToOpen);
    };

    rsx! {
        div { class: "flex items-center justify-center h-full",
            div { class: "max-w-md w-full p-8 bg-app-panel rounded-lg shadow-lg border border-app-border",
                h1 { class: "text-2xl font-bold text-app-text mb-6 text-center", "Welcome to Footnote" }

                div { class: "space-y-3",
                    button {
                        class: "w-full px-4 py-3 bg-app-primary text-white rounded-md hover:bg-app-primary-hover font-medium",
                        onclick: handle_create,
                        "Create"
                    }
                    button {
                        class: "w-full px-4 py-3 bg-app-hover text-app-text-secondary border border-app-border-subtle rounded-md hover:bg-app-hover font-medium",
                        onclick: handle_join,
                        "Join"
                    }
                    button {
                        class: "w-full px-4 py-3 bg-app-hover text-app-text-secondary border border-app-border-subtle rounded-md hover:bg-app-hover font-medium",
                        onclick: handle_open,
                        "Open"
                    }
                }
            }
        }
    }
}
