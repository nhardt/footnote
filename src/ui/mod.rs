mod app;
mod config;
mod markdown;

pub fn launch() {
    dioxus::launch(app::App);
}
