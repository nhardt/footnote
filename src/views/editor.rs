use std::path::PathBuf;

use dioxus::prelude::*;

use crate::{components::FileSearch, context::VaultContext};

#[component]
pub fn Editor() -> Element {
    let vault_ctx = use_context::<VaultContext>();
    let vault_path = vault_ctx.get_vault().unwrap_or_default();
    let mut current_file = use_signal(|| None::<PathBuf>);

    rsx! {
        div { class: "max-w-4xl mx-auto p-6 h-full flex flex-col gap-4",
            FileSearch {
                search_path: vault_path.clone(),
                on_select: move |path| {
                    current_file.set(Some(path));
                }
            }

            if let Some(file) = current_file() {
                div { "File: {file.display()}" }
            }
        }
    }
}
