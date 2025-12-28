use crate::{components::directoy_browser::DirectoryBrowser, context::VaultContext, Route};
use dioxus::prelude::*;
use footnote::{
    model::{note::Note, vault::Vault},
    platform::get_app_dir,
};
use std::{fmt::format, path::PathBuf};

#[component]
pub fn NoteBrowser() -> Element {
    let vault_ctx = use_context::<VaultContext>();
    let vault_path = match vault_ctx.get_vault() {
        Some(path) => path.clone(),
        None => return rsx! { "Error loading vault" },
    };
    let vault = Vault::new(&vault_path)?;
    let nav = navigator();

    let mut err_message = use_signal(|| String::new());

    rsx! {
        DirectoryBrowser {
            base_path: vault_path,
            only_directories: false,
            on_select: move |file_path:PathBuf| {
                let path = file_path.display().to_string().clone();
                let encoded = urlencoding::encode(&path);
                nav.push(Route::NoteView { file_path: encoded.into_owned() });
            },
            on_file_create: move |file_path:PathBuf| {
                if let Err(e) = vault.note_create(&file_path, "New Note") {
                    err_message.set(format!("{}", e));
                };
            },
            on_cancel: move|_| info!("on_cancel"),
            action_label: "Select Note".to_string(),
        }
        label { "{err_message}" }
    }
}
