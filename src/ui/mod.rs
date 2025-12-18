mod app;
mod config;
mod context;
mod markdown;
mod plaintext;
mod screens;
use tracing::Level;

pub use config::AppConfig;
pub use context::VaultContext;

pub fn launch() {
    dioxus::logger::init(Level::DEBUG).expect("failed to init logger");
    tracing::trace!("trace level logging enabled");
    tracing::debug!("debug level logging enabled");
    tracing::info!("info level logging enabled");
    tracing::warn!("warn level logging enabled");
    tracing::error!("error level logging enabled");
    dioxus::launch(app::App);
}
