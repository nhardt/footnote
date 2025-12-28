use crate::{components::directoy_browser::DirectoryBrowser, context::VaultContext, Route};
use dioxus::prelude::*;
use footnote::platform::get_app_dir;
use std::path::PathBuf;

#[component]
pub fn NoteBrowser() -> Element {
    let vault_ctx = use_context::<VaultContext>();
    let vault_path = match vault_ctx.get_vault() {
        Some(path) => path.clone(),
        None => return rsx! { "Error loading vault" },
    };

    let nav = navigator();

    rsx! {
        DirectoryBrowser {
            base_path: vault_path,
            only_directories: false,
            on_select: move |file_path:PathBuf| {
                let path = file_path.display().to_string().clone();
                let encoded = urlencoding::encode(&path);
                nav.push(Route::NoteView { file_path: encoded.into_owned() });
            },
            on_cancel: move|_| info!("on_cancel"),
            action_label: "Select Note".to_string(),
        }
    }
}
