use dioxus::prelude::*;
use std::path::PathBuf;
use std::fs;
use urlencoding;

use crate::context::VaultContext;

#[component]
pub fn Edit(file_path: String) -> Element {
    let vault_ctx = use_context::<VaultContext>();
    let vault_path = vault_ctx.get_vault().unwrap_or_default();

    let path = vault_path.join(&file_path);
    let mut content = use_signal(String::new);
    let mut error_msg = use_signal(|| None::<String>);
    let mut success_msg = use_signal(|| None::<String>);

    // Load file content on mount
    let path_for_load = path.clone();
    use_effect(move || {
        let path = path_for_load.clone();
        if let Ok(file_content) = fs::read_to_string(&path) {
            content.set(file_content);
        } else {
            error_msg.set(Some(format!("Failed to read file: {}", path.display())));
        }
    });

    let path_for_save = path.clone();
    let save_file = move |_| {
        let path = path_for_save.clone();
        match fs::write(&path, content()) {
            Ok(_) => {
                success_msg.set(Some("Saved successfully".to_string()));
                error_msg.set(None);
            }
            Err(e) => {
                error_msg.set(Some(format!("Failed to save: {}", e)));
                success_msg.set(None);
            }
        }
    };

    rsx! {
        div {
            div {
                {path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Untitled")}
            }
            div {
                button {
                    onclick: save_file,
                    "Save"
                }
                Link {
                    to: format!("/browse?file={}", urlencoding::encode(&file_path)),
                    "View"
                }
            }
        }

        if let Some(error) = error_msg() {
            div { "{error}" }
        }

        if let Some(success) = success_msg() {
            div { "{success}" }
        }

        textarea {
            value: "{content}",
            oninput: move |evt| content.set(evt.value()),
            style: "width: 100%; height: 80vh; font-family: monospace;"
        }
    }
}
