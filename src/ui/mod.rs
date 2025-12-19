mod app;
mod components;
mod config;
mod context;
mod plaintext;
mod screens;
use std::path::PathBuf;

use tracing::Level;

pub use config::AppConfig;
pub use context::VaultContext;

#[derive(Clone, Copy, PartialEq)]
pub enum Screen {
    Editor,
    Contacts,
    Profile,
}

#[derive(Clone, PartialEq)]
pub struct FootnoteFile {
    pub path: PathBuf,
    pub filename: String,
    pub content: String,
    pub share_with: Vec<String>,
    pub footnotes: Vec<crate::core::note::Footnote>,
}

pub fn launch() {
    dioxus::logger::init(Level::DEBUG).expect("failed to init logger");
    tracing::trace!("trace level logging enabled");
    tracing::debug!("debug level logging enabled");
    tracing::info!("info level logging enabled");
    tracing::warn!("warn level logging enabled");
    tracing::error!("error level logging enabled");
    dioxus::launch(app::App);
}
