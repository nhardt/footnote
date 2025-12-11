mod app;
mod config;
mod markdown;
mod context;
mod screens;

pub use config::AppConfig;
pub use context::VaultContext;

pub fn launch() {
    dioxus::launch(app::App);
}
