use crate::components::DirectoryBrowser;
use crate::context::VaultContext;
use crate::Route;
use dioxus::prelude::*;
use std::path::PathBuf;

#[component]
pub fn VaultHome() -> Element {
    let nav = navigator();

    rsx! {
        div { class: "vault-home",
            div { class: "card",
                h1 { "Welcome to Footnote" }

                div { class: "button-group",
                    button {
                        onclick: move |_| {
                            nav.replace(Route::VaultCreate {});
                        },
                        "Create Vault"
                    }

                    button {
                        onclick: move |_| {
                            nav.replace(Route::VaultOpen {});
                        },
                        "Open Vault"
                    }

                    button {
                        onclick: move |_| {
                            nav.replace(Route::VaultJoin {});
                        },
                        "Join Vault"
                    }
                }
            }
        }
    }
}

#[component]
pub fn VaultCreate() -> Element {
    let mut vault_ctx = use_context::<VaultContext>();
    let nav = navigator();

    rsx! {
        DirectoryBrowser {
            action_label: "Create Here".to_string(),
            is_valid: move |path: PathBuf| !path.join(".footnotes").exists(),
            on_select: move |path| {
                vault_ctx.set_vault(path);
                nav.replace(Route::Browse { file_path:"home.md".to_string() });
            },
            on_cancel: move |_| {
                nav.replace(Route::VaultHome {});
            }
        }
    }
}

#[component]
pub fn VaultOpen() -> Element {
    let mut vault_ctx = use_context::<VaultContext>();
    let nav = navigator();

    rsx! {
        DirectoryBrowser {
            action_label: "Open".to_string(),
            is_valid: move |path: PathBuf| path.join(".footnotes").exists(),
            on_select: move |path| {
                vault_ctx.set_vault(path);
                nav.replace(Route::Browse { file_path:"home.md".to_string() });
            },
            on_cancel: move |_| {
                nav.replace(Route::VaultHome {});
            }
        }
    }
}
