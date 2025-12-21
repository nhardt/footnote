use std::path::PathBuf;

use dioxus::prelude::*;

use crate::{components::{FileSearch, TitleInput}, context::VaultContext};

#[component]
pub fn Editor() -> Element {
    let vault_ctx = use_context::<VaultContext>();
    let vault_path = vault_ctx.get_vault().unwrap_or_default();
    let mut current_file = use_signal(|| None::<PathBuf>);

    // Get title from filename (without .md extension)
    let title = current_file().and_then(|file| {
        file.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
    });

    rsx! {
        div { class: "editor",
            FileSearch {
                search_path: vault_path.clone(),
                on_select: move |path| {
                    current_file.set(Some(path));
                }
            }

            if let Some(title_text) = title {
                TitleInput {
                    value: title_text.clone(),
                    on_change: move |new_title| {
                        if let Some(file) = current_file() {
                            let new_path = file.parent().unwrap().join(format!("{}.md", new_title));
                            if std::fs::rename(&file, &new_path).is_ok() {
                                current_file.set(Some(new_path));
                            }
                        }
                    }
                }
            }
        }
    }
}
