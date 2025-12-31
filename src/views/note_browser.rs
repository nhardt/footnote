use crate::{components::directoy_browser::DirectoryBrowser, context::VaultContext, Route};
use dioxus::{logger, prelude::*};
use footnote::{
    model::{note::Note, vault::Vault},
    platform::get_app_dir,
};
use std::{fmt::format, path::PathBuf};

#[component]
pub fn NoteBrowser() -> Element {
    tracing::trace!("NoteBrowser re-render");
    let vault_ctx = use_context::<VaultContext>();
    let vault_path = use_memo(move || vault_ctx.get_vault().unwrap().clone());

    let nav = navigator();

    rsx! {
        DirectoryBrowser {
            base_path: vault_path(),
            only_directories: false,
            on_select: move |file_path:PathBuf| {
                let path = file_path.display().to_string().clone();
                let encoded = urlencoding::encode(&path);
                nav.push(Route::NoteView { file_path: encoded.into_owned() });
            },
            on_file_create: move |file_path:PathBuf| {
                let vault_path = vault_ctx.get_vault().expect("vault path should be valid");
                let vault = Vault::new(&vault_path).expect("should be able to create vault from vault path");
                if let Err(e) = vault.note_create(&file_path, "New Note") {
                    eprintln!("error creating note: {}", e);
                };
            },
            on_cancel: move|_| info!("on_cancel"),
            action_label: "Select Note".to_string(),
        }
    }
}
