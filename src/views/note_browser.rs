use crate::{components::directoy_browser::DirectoryBrowser, context::AppContext, Route};
use crate::{
    model::{note::Note, vault::Vault},
    platform::get_app_dir,
};
use dioxus::{logger, prelude::*};
use std::{fmt::format, path::PathBuf};

#[component]
pub fn NoteBrowser() -> Element {
    tracing::trace!("NoteBrowser re-render");
    let app_context = use_context::<AppContext>();
    let vault = app_context.vault.read().clone();
    let nav = navigator();

    rsx! {
        DirectoryBrowser {
            base_path: vault.base_path(),
            only_directories: false,
            on_select: move |file_path:PathBuf| {
                let path = file_path.display().to_string().clone();
                let encoded = urlencoding::encode(&path);
                nav.push(Route::NoteView { file_path: encoded.into_owned() });
            },
            on_file_create: move |file_path:PathBuf| {
                let vault = vault.clone();
                if let Err(e) = vault.note_create(&file_path, "New Note") {
                    eprintln!("error creating note: {}", e);
                };
            },
            on_cancel: move|_| info!("on_cancel"),
            action_label: "Select Note".to_string(),
        }
    }
}
