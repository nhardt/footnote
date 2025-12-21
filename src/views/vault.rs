use crate::components::Button;
use crate::components::DirectoryBrowser;
use crate::context::VaultContext;
use crate::Route;
use dioxus::prelude::*;
use std::path::PathBuf;

#[component]
pub fn VaultHome() -> Element {
    let nav = navigator();

    rsx! {
        div { class: "flex items-center justify-center h-full",
            div { class: "max-w-md w-full p-8 bg-zinc-800 rounded-lg shadow-lg border border-zinc-700",
                h1 { class: "text-2xl font-bold text-zinc-100 mb-6 text-center", "Welcome to Footnote" }

                div { class: "flex flex-col gap-8",
                    Button{
                        onclick: move |_| {
                            nav.replace(Route::VaultCreate {});
                        },
                       "Create Vault"
                    }

                    Button{
                        onclick: move |_| {
                            nav.replace(Route::VaultOpen {});
                        },
                        "Open Vault"
                    }

                    Button{
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
                nav.replace(Route::Editor {});
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
                nav.replace(Route::Editor {});
            },
            on_cancel: move |_| {
                nav.replace(Route::VaultHome {});
            }
        }
    }
}

#[component]
pub fn VaultJoin() -> Element {
    let mut vault_ctx = use_context::<VaultContext>();
    let nav = navigator();

    rsx! {
        DirectoryBrowser {
            action_label: "Local Directory To Mirror To".to_string(),
            is_valid: move |path: PathBuf| path.join(".footnotes").exists(),
            on_select: move |path| {
                vault_ctx.set_vault(path);
                nav.replace(Route::Editor {});
            },
            on_cancel: move |_| {
                nav.replace(Route::VaultHome {});
            }
        }
    }
}
